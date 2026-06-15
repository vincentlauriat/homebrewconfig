# Architecture

> Miroir français de [`ARCHITECTURE_EN.md`](ARCHITECTURE_EN.md) (source de
> vérité). Les deux fichiers sont édités dans le même changement.

Ce document décrit l'architecture interne de **homebrewconfig**, écrit en Rust.
L'outil a deux visages partageant le même cœur :

- une **TUI** interactive (ratatui/crossterm) pour parcourir et éditer les
  réglages ;
- une **CLI** non interactive (mode *batch*) pour le scripting.

Tous deux éditent les variables d'environnement Homebrew dans le profil shell de
l'utilisateur.

## Vue d'ensemble

```
                          ┌──────────────────────────────┐
                          │            main.rs           │
                          │  parse_args → Cli            │
                          │  is_batch ? run_batch : TUI  │
                          └───────┬───────────────┬──────┘
              batch (CLI)         │               │  interactif (TUI)
        ┌───────────────┬─────────┘               └───────┬───────────────┐
        ▼               ▼                                 ▼               ▼
  ┌───────────┐   ┌───────────┐                     ┌───────────┐   ┌───────────┐
  │ preset.rs │   │ report.rs │                     │  boucle   │   │   ui.rs   │
  │ TOML I/O  │   │ JSON out  │                     │ d'events  │──▶│  rendu    │
  └─────┬─────┘   └─────┬─────┘                     └─────┬─────┘   │ (lecture) │
        │               │      ┌───────────┐              │         └───────────┘
        │               │      │  brew.rs  │              │
        │               │      │ brew env  │              │
        └───────┬───────┴──────┴─────┬─────┘              │
                ▼                    ▼                    ▼
            ┌──────────────────────────────────────────────┐
            │                   app.rs                      │
            │   App, Setting, Mode, SettingKind — état &    │
            │   logique métier (catalogue, filtre, profil)  │
            └───────────────────────┬──────────────────────┘
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
- **View** → `ui.rs` est une fonction de rendu qui ne fait que lire `&App`.
- **Update** → `main.rs` capture les événements clavier et mute `App`.
- **Effets de bord** → `config.rs` isole l'écriture du profil ; `brew.rs` le seul
  sous-processus ; `preset.rs` / `report.rs` la sérialisation.

Aucune dépendance circulaire : tous les modules dépendent de `app.rs`, jamais
l'inverse. `app.rs` ne dépend que de `config.rs` (pour la détection du bloc).

## Modules

### `main.rs` — point d'entrée, CLI & boucle d'événements

- **`parse_args` → `Cli`** : analyse les arguments en une structure `Cli`
  (profil, `sets`/`unsets`, presets, drapeaux `apply`/`dry_run`/`list`/`json`/
  `brew_env`). `Cli::is_batch()` décide du mode.
- **Mode batch** (`run_batch`) : si un drapeau non interactif est présent, on
  n'ouvre pas la TUI. Ordre : `--brew-env` (autonome) → import preset (baseline)
  → `--set`/`--unset` (override) → export preset → `--list` → `--json` →
  `--dry-run` → `--apply`. Les erreurs sortent avec un code ≠ 0.
- **Mode TUI** :
  - **Setup/teardown terminal** : raw mode + écran alternatif ; la restauration
    est garantie **avant** toute propagation d'erreur.
  - **Boucle `run`** : expiration des messages (`MESSAGE_TIMEOUT` = 3 s),
    `terminal.draw`, `event::poll(100ms)`, filtre `KeyEventKind::Press`.
  - **Routage clavier** par mode : `handle_normal`, `handle_editing`,
    `handle_confirm`, `handle_filter`. Curseur d'édition *char-aware*
    (`char_to_byte`) pour l'UTF-8.

### `app.rs` — état & logique métier

Types clés :

- **`SettingKind`** : `Bool { inverted }`, `Str`, `Num`. Le flag `inverted` est
  central : beaucoup de variables Homebrew sont des **négations**
  (`HOMEBREW_NO_ANALYTICS`). L'UI montre la *fonctionnalité* (« Analytics ON »)
  tandis que la variable code l'inverse ; `inverted: true` fait la traduction
  dans les deux sens (lecture env ↔ affichage, affichage ↔ export).
- **`Setting`** : métadonnées statiques (`name`, `env_var`, `description`,
  `category`, `kind`) + valeurs (`bool_val`, `str_val`, `num_val`) + `modified`
  + `default_hint` (défaut Homebrew affiché) + `is_path` (chemin à valider). Des
  builders chaînables — `with_default`, `with_path_flag` — n'enrichissent que les
  entrées concernées.
- **`Mode`** : `Normal`, `Editing`, `Confirming`, `Filtering`.
- **`App`** : `Vec<Setting>`, index sélectionné, `ListState` ratatui, mode,
  buffer d'édition, **filtre** courant, **profil shell** + **candidats**, message
  optionnel `(texte, is_error)`, `show_help`.

Logique notable :

- **`apply_env_value`** (pur) : conversion d'une `Option<String>` brute vers la
  valeur typée, séparée de `read_from_env` pour être testable sans toucher
  l'environnement du process. Applique l'inversion.
- **`build_settings`** : catalogue des **22 réglages** (source de vérité unique).
- **`set_var`** : applique une valeur par variable d'environnement — utilisé par
  la CLI batch (`--set`/`--unset`/presets).
- **Filtre** : `visible()` renvoie les indices correspondant au filtre (nom, var,
  catégorie, description) ; navigation et `sync_list_state` opèrent sur cet
  ensemble visible.
- **Profil** : `profile_candidates()` liste les fichiers candidats par shell ;
  `pick_profile(candidats, has_block, exists)` (prédicats injectés, donc
  testable) choisit le fichier contenant déjà le bloc, sinon le premier existant,
  sinon le préféré ; `cycle_profile` cycle la cible dans la TUI.
- **`path_status(exists)`** (prédicat injecté) : pour un réglage chemin renseigné,
  indique s'il existe.

### `ui.rs` — rendu (lecture seule)

- `render` découpe l'écran en **header** (4) / **body** (min) / **status bar** (3).
  Le body est scindé : liste des réglages (55 %) + détail (45 %).
- Popups via `Clear` + `centered_rect` : édition, **confirmation+preview**
  (`render_confirm_popup`, bloc colorisé), aide.
- Le header affiche la version (`CARGO_PKG_VERSION`), le profil cible et un
  compteur **« ● N unsaved »**. Le détail montre la valeur par défaut Homebrew et
  le statut d'existence des chemins.
- La liste et le détail respectent le **filtre** (`(no matches)` géré).
- **Invariant** : ne mute jamais l'état métier (`&mut App` seulement pour
  `ListState`, exigé par `render_stateful_widget`).

### `config.rs` — persistance du profil shell

Seul module qui écrit le profil. Garanties :

- **Bloc délimité** `# homebrewconfig BEGIN/END`, **idempotent** (`replace_block`
  remplace en place, sinon ajoute en fin de fichier, préservant le reste).
- **Backup** : copie vers `<profil>.bak` avant écrasement ; une application sans
  changement est un **no-op**.
- **`generate_block`** : n'exporte que les valeurs non-défaut (inversion gérée).
- **`escape_value`** : protège `\` et `"`. **`file_contains_block`** : détecte
  notre bloc (utilisé par la sélection de profil).

### `preset.rs` — presets TOML (serde + toml)

- **`export_preset`** : sérialise les réglages non-défaut en table `[settings]`
  (`HOMEBREW_* = "value"`).
- **`parse_preset`** : lit un TOML en paires `(env_var, raw)`.
- Un preset sert de **baseline** ; les `--set`/`--unset` l'emportent ensuite.

### `report.rs` — sortie JSON (serde_json)

- **`state_json`** : tableau JSON typé par kind (`bool`/`number`/`string`) de
  tous les réglages, avec `value`, `modified` et `default`.

### `brew.rs` — lecture de l'environnement Homebrew

- **`brew_environment`** : lance `brew environment --shell=bash` (seul
  sous-processus de l'app) ; `None` si `brew` est introuvable.
- **`parse_brew_env`** (pur) : extrait les assignations `HOMEBREW_*`.
- Dégradation gracieuse : `--brew-env` ne modifie rien.

### `theme.rs` — palettes de couleurs

- **`Theme`** (`Copy`) : toutes les couleurs de l'UI (primary, accent, on/off,
  bg, etc.). Le rendu source **toutes** ses couleurs depuis le thème actif, donc
  l'apparence est interchangeable à l'exécution.
- **`THEMES`** : liste statique (`brew`, `midnight`, `forest`, `rose`, `mono`),
  le premier étant le défaut. `index_of`/`names` pour la résolution par nom.

### `appconfig.rs` — préférences utilisateur (serde + toml)

- Persiste les préférences de l'outil lui-même (distinctes des variables
  Homebrew gérées) dans `config.toml` sous le répertoire de config utilisateur
  (`dirs::config_dir()`). Pour l'instant : le thème choisi.
- `load` tolère l'absence/invalidité (retombe sur les défauts) ; `save` est
  best-effort (un répertoire en lecture seule ne casse pas l'UI).

## Flux de données

### Démarrage (TUI)
```
main() → parse_args() → App::with_profile(profil)
       → build_settings() → read_from_env() (×22)
       → profile_candidates() / pick_profile() → boucle run()
```

### Application (`a` → confirmation → `y`)
```
handle_normal('a') → Mode::Confirming → render_confirm_popup (preview)
handle_confirm('y') → config::apply_config(&app)
   → read profil → generate_block() → replace_block()
   → backup <profil>.bak → fs::write() → reset des dirty flags
```

### Mode batch (CLI)
```
parse_args() → Cli::is_batch() == true → run_batch()
   → import preset → set/unset → export preset → list/json/dry-run/apply
```

## Décisions de conception

- **Catalogue statique en code** : zéro parsing, métadonnées `&'static str` sans
  coût, la liste des variables Homebrew évolue lentement.
- **Bloc géré idempotent + backup** : on ne possède que notre section, le reste
  du profil reste intact, et toute écriture est réversible.
- **Logique pure injectable** (`apply_env_value`, `pick_profile`, `path_status`,
  `parse_brew_env`, `generate_block(&[Setting])`) : testable sans I/O — d'où une
  suite de **47 tests** sans toucher au terminal ni au disque.
- **Cœur partagé TUI/CLI** : la même logique `App`/`Setting` sert les deux modes.
- **Teardown avant propagation d'erreur** : robustesse du terminal.

## Dépendances

| Crate | Rôle |
|-------|------|
| `ratatui` 0.29 | Widgets et layout TUI |
| `crossterm` 0.28 | Backend terminal (raw mode, événements, écran alternatif) |
| `dirs` 5 | Résolution du répertoire home multi-plateforme |
| `serde` 1 + `toml` 0.8 | Sérialisation des presets TOML |
| `serde_json` 1 | Sortie `--json` |

## Points d'extension

- **Ajouter un réglage** : une entrée dans `App::build_settings` (avec
  éventuellement `.with_default(...)` / `.with_path_flag()`). Rendu, lecture env,
  export, presets et JSON le prennent en charge automatiquement.
- **Nouveau type de réglage** : étendre `SettingKind` puis compléter les `match`
  (`value_display`, `apply_env_value`, `start_editing`, `confirm_edit`,
  `generate_block`, `report`, le rendu).
- **Nouveau shell** : une branche dans `App::profile_candidates`.
- **Nouvelle commande CLI** : un champ dans `Cli`, une branche dans `parse_args`,
  un traitement dans `run_batch`.
- **Nouveau thème** : une entrée dans `theme::THEMES`. Cyclage, `--theme` et la
  persistance le prennent en charge automatiquement.
