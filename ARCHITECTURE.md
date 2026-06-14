# Architecture

Ce document décrit l'architecture interne de **homebrewconfig**, une application TUI (Terminal User Interface) écrite en Rust qui édite les variables d'environnement de Homebrew dans le profil shell de l'utilisateur.

## Vue d'ensemble

```
┌──────────────────────────────────────────────────────────────┐
│                          main.rs                             │
│   Boucle d'événements · setup/teardown du terminal           │
│   Routage clavier (Mode::Normal / Mode::Editing)             │
└───────────────┬───────────────────────────┬──────────────────┘
                │                           │
                ▼                           ▼
        ┌───────────────┐          ┌──────────────────┐
        │    app.rs     │          │      ui.rs       │
        │  État & logique│◀────────│  Rendu ratatui   │
        │  (App, Setting)│  &App   │  (lecture seule) │
        └───────┬───────┘          └──────────────────┘
                │ &App
                ▼
        ┌───────────────┐
        │   config.rs   │
        │  I/O profil   │
        │  shell (write)│
        └───────────────┘
```

L'application suit un modèle proche de **Model–View–Update** :

- **Model** → `App` et `Setting` (`app.rs`) détiennent tout l'état.
- **View** → `ui.rs` est une fonction pure de rendu qui ne fait que lire `&App`.
- **Update** → `main.rs` capture les événements clavier et mute `App` via ses méthodes.
- **Effets de bord** → `config.rs` isole la seule écriture sur disque (le profil shell).

Aucune dépendance circulaire : `ui.rs` et `config.rs` dépendent de `app.rs`, jamais l'inverse.

## Modules

### `main.rs` — point d'entrée & boucle d'événements

Responsabilités :

- **Setup terminal** : `enable_raw_mode`, passage en écran alternatif (`EnterAlternateScreen`), capture souris.
- **Teardown garanti** : la restauration du terminal (`disable_raw_mode`, `LeaveAlternateScreen`) est exécutée **avant** de propager une éventuelle erreur, pour ne jamais laisser le terminal dans un état cassé.
- **Boucle principale** (`run`) :
  - Gère l'expiration des messages de statut (`MESSAGE_TIMEOUT` = 3 s).
  - Redessine via `terminal.draw(|f| ui::render(f, app))`.
  - Sonde les événements avec `event::poll(100ms)` (tick non bloquant, nécessaire pour faire expirer les messages même sans frappe).
  - Filtre les répétitions clavier : ne traite que `KeyEventKind::Press`.
- **Routage clavier** : `handle_normal` (navigation, toggle, apply, quit) et `handle_editing` (saisie de texte dans la popup). La gestion du curseur est *char-aware* via `char_to_byte` pour rester correcte en UTF-8.

### `app.rs` — état & logique métier

Types clés :

- **`SettingKind`** : `Bool { inverted }`, `Str`, `Num`.
  - Le flag `inverted` est central : beaucoup de variables Homebrew sont des **négations** (`HOMEBREW_NO_ANALYTICS`). Dans l'UI on présente la *fonctionnalité* (« Analytics ON ») alors que la variable d'environnement code l'inverse. `inverted: true` fait la traduction dans les deux sens (lecture env ↔ affichage, affichage ↔ export).
- **`Setting`** : un réglage = métadonnées statiques (`name`, `env_var`, `description`, `category`, `kind`) + valeurs courantes (`bool_val`, `str_val`, `num_val`) + `modified` (dirty flag).
- **`Mode`** : `Normal` ou `Editing`.
- **`App`** : agrège `Vec<Setting>`, l'index sélectionné, l'état de la liste ratatui (`ListState`), le mode courant, le buffer d'édition (`input` + `cursor_pos`), le chemin du profil shell, un message optionnel `(texte, is_error)` et le flag `show_help`.

Logique notable :

- **`read_from_env`** : initialise chaque réglage à partir de l'environnement courant au lancement (et lors d'un `reset`). Applique l'inversion pour les booléens.
- **`build_settings`** : déclare le catalogue complet des 22 réglages (source de vérité unique). Ajouter un réglage = ajouter une entrée ici.
- **`compute_list_index`** : la liste affichée intercale des **en-têtes de catégorie** et des lignes vides ; cette fonction convertit l'index logique d'un réglage en index visuel dans la `List` ratatui, pour garder la surbrillance alignée.
- **`detect_shell_profile`** : choisit le fichier cible selon `$SHELL` (zsh → `~/.zprofile`, fish → `~/.config/fish/config.fish`, sinon → `~/.bash_profile`).

### `ui.rs` — rendu (lecture seule)

- Fonction racine `render(f, app)` qui découpe l'écran en trois bandes verticales : **header** (4), **body** (min), **status bar** (3).
- Le body est découpé horizontalement : liste des réglages (55 %) + panneau de détail (45 %).
- Popups par-dessus via `Clear` + `centered_rect` : édition (`render_edit_popup`) et aide (`render_help_popup`).
- Palette « Homebrew » centralisée en constantes (`BREW_GOLD`, `BREW_AMBER`, `ON_COLOR`, etc.).
- `truncate` coupe proprement les valeurs longues en UTF-8 avec une ellipse.
- **Invariant** : ce module ne mute jamais l'état métier (il reçoit `&App` ; `&mut App` n'est utilisé que pour `ListState`, exigé par `render_stateful_widget`).

### `config.rs` — persistance du profil shell

C'est le seul module qui écrit sur le disque. Garanties :

- **Bloc délimité** : tout est encadré par `# homebrewconfig BEGIN` / `# homebrewconfig END`.
- **Idempotence** (`replace_block`) : si le bloc existe déjà, il est remplacé *en place* ; sinon il est ajouté en fin de fichier. Relancer l'outil ne crée jamais de doublon et préserve le reste du profil.
- **Création de parents** : `create_dir_all` pour le cas fish (`~/.config/fish/`).
- **Génération** (`generate_block`) : n'exporte que les valeurs « non-défaut » — un booléen à sa valeur par défaut ou une chaîne vide ne produit aucune ligne, gardant le profil minimal. Inversion gérée ici aussi.
- **Échappement** (`escape_value`) : protège `\` et `"` dans les valeurs chaînes.

## Flux de données

### Démarrage
```
main() → App::new() → build_settings() → read_from_env() (×22)
       → detect_shell_profile() → boucle run()
```

### Frappe clavier (mode normal)
```
event::read() → handle_normal() → App::{select_*, toggle_current, start_editing, reset}
              → ui::render() au tick suivant
```

### Application (`a`)
```
handle_normal('a') → config::apply_config(&app)
   → read profil existant → generate_block() → replace_block() → fs::write()
   → reset des dirty flags → message de confirmation
```

## Décisions de conception

- **Catalogue statique en code** plutôt qu'un fichier de config : zéro dépendance de parsing, les métadonnées (`&'static str`) sont sans coût, et la liste des variables Homebrew évolue lentement.
- **Bloc géré idempotent** plutôt que réécriture complète du profil : on ne s'approprie que notre section, le reste du fichier utilisateur reste intact et éditable à la main.
- **Séparation View/Update stricte** : le rendu est rejouable et testable indépendamment de la logique.
- **Teardown avant propagation d'erreur** : robustesse du terminal en cas de panic/erreur.

## Dépendances

| Crate | Rôle |
|-------|------|
| `ratatui` 0.29 | Widgets et layout TUI |
| `crossterm` 0.28 | Backend terminal (raw mode, événements, écran alternatif) |
| `dirs` 5 | Résolution du répertoire home multi-plateforme |

## Points d'extension

- **Ajouter un réglage** : une entrée dans `App::build_settings`. Le rendu, la lecture env et l'export le prennent en charge automatiquement.
- **Nouveau type de réglage** (ex. énumération) : étendre `SettingKind` puis compléter les `match` dans `value_display`, `read_from_env`, `start_editing`, `confirm_edit`, `generate_block` et le rendu.
- **Nouveau shell** : une branche dans `detect_shell_profile`.
