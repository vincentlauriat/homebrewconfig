# TODOS

Suivi de l'avancement de **homebrewconfig**. Cases cochées = terminé.

## ✅ Fait (v0.1.0)

- [x] Boucle TUI avec ratatui + crossterm (raw mode, écran alternatif, teardown sûr)
- [x] Catalogue des 22 réglages Homebrew (`build_settings`)
- [x] Trois types de réglages : booléen (avec inversion), chaîne, nombre
- [x] Lecture de l'environnement courant au démarrage et au `reset`
- [x] Navigation clavier (`↑/↓`, `j/k`), regroupement par catégorie
- [x] Toggle des booléens (`Space`)
- [x] Édition inline des chaînes/nombres avec popup et curseur UTF-8
- [x] Validation de saisie (nombres : chiffres uniquement)
- [x] Écriture idempotente du bloc dans le profil shell (`apply_config`)
- [x] Détection du shell (zsh / bash / fish)
- [x] Échappement des valeurs chaînes (`\` et `"`)
- [x] Panneau de détail + barre de statut + popup d'aide
- [x] Messages de statut avec expiration (3 s)
- [x] Dirty flags + avertissement « changements non sauvegardés » à la sortie
- [x] README.md

## 🔜 Court terme

- [x] **Tests unitaires** pour `config.rs` (`replace_block`, `generate_block`, `escape_value`) — 12 tests, logique pure couverte ; `generate_block` découplé pour prendre `&[Setting]`
- [x] **Backup** du profil avant écriture (`<profil>.bak`) pour rollback
- [x] **Confirmation** avant `apply` (popup « Apply changes? » avec `y/n`)
- [x] **Diff preview** : le bloc qui sera écrit est affiché et colorisé dans la popup de confirmation
- [x] **Tests** de la conversion `read_from_env` : logique extraite dans `apply_env_value(Option<String>)` (pure) et couverte par 5 tests (inversion, défauts, parsing num)
- [x] Gestion de la **valeur par défaut visible** : affichée dans le détail pour les réglages à défaut notable (Cleanup Age, Curl Retries, Install Badge, Editor)
- [x] **Indicateur global** « ● N unsaved » dans le header tant que des changements ne sont pas appliqués

## 📦 Moyen terme

- [x] **CI GitHub Actions** : `cargo fmt --check`, `cargo clippy -D warnings`, `cargo build`, `cargo test` (`.github/workflows/ci.yml`)
- [x] **Recherche / filtre** des réglages (`/` ; matche name/env_var/category/description, navigation et rendu filtrés, 5 tests)
- [ ] **Détection du profil multi-source** : aussi lire `.zshrc` / `.bashrc` si l'export y est déjà
- [x] **Cible de profil configurable** : auto-détection (fichier contenant déjà le bloc → existant → préféré), flag `--profile <PATH>`, et touche `p` pour cycler dans l'UI
- [ ] **Mode CLI non interactif** (`--set HOMEBREW_NO_ANALYTICS=1`, `--apply`) pour scripting
- [ ] **Export/import** de presets (profils partagés en JSON/TOML)
- [ ] Support **Windows** (si pertinent) ou message clair de non-support

## 🎯 Long terme / idées

- [ ] Lire les variables réellement reconnues via `brew --env` pour rester synchro avec Homebrew
- [ ] **Validation sémantique** des chemins (Cache/Cellar/Logs/Temp existent ?)
- [ ] **Thèmes** de couleurs configurables
- [ ] Publication sur **crates.io** et formule **Homebrew** (`brew install homebrewconfig`)
- [ ] Page de **man** / complétion shell

## 🐛 Dette technique / à surveiller

- [ ] `cursor_pos` dans la popup d'édition ne gère pas le scroll horizontal si la valeur dépasse la largeur du champ
- [ ] Pas de gestion d'erreur si `$SHELL` est inhabituel (fallback silencieux sur bash)
- [x] ~~`detect_shell_profile` suppose `~/.zprofile`~~ → résolu : `.zshrc` préféré, détection du fichier contenant déjà le bloc, override `--profile` + cycle `p`
