# TODOS

Suivi de l'avancement de **homebrewconfig**. Cases cochées = terminé.

## ✅ Livré en v0.1.0

- [x] Boucle TUI ratatui + crossterm (raw mode, écran alternatif, teardown sûr)
- [x] Catalogue des 22 réglages Homebrew (`build_settings`)
- [x] Trois types de réglages : booléen (avec inversion), chaîne, nombre
- [x] Lecture de l'environnement courant au démarrage et au `reset`
- [x] Navigation clavier (`↑/↓`, `j/k`), regroupement par catégorie
- [x] Toggle (`Space`) + édition inline (popup, curseur UTF-8, validation des nombres)
- [x] Écriture idempotente du bloc dans le profil shell
- [x] Détection du shell (zsh / bash / fish)
- [x] Échappement des valeurs chaînes, panneau de détail, aide, messages

## ✅ Livré en v0.2.0 (publiée sur crates.io)

### Jalon 1 — robustesse
- [x] Tests unitaires de la logique pure (`config.rs`, `apply_env_value`)
- [x] Backup `<profil>.bak` avant écriture + no-op si rien ne change
- [x] Confirmation + **aperçu colorisé** avant `apply`
- [x] CI GitHub Actions (`fmt --check`, `clippy -D warnings`, `build`, `test`)

### Jalon 2 — ergonomie
- [x] Recherche / filtre incrémental (`/`)
- [x] Valeur par défaut Homebrew dans le détail
- [x] Indicateur « ● N unsaved » dans le header
- [x] Cible de profil configurable (auto-détection, `--profile`, cycle `p`) — résout `.zprofile` vs `.zshrc`

### Jalon 3 — scriptabilité
- [x] CLI non interactive : `--set`, `--unset`, `--apply`, `--dry-run`, `--list`
- [x] Presets TOML : `--export-preset` / `--import-preset`
- [x] Sortie `--json` typée de l'état courant

### Jalon 4 — distribution
- [x] `--brew-env` : lecture de l'environnement effectif via `brew`
- [x] Validation d'existence des chemins (Cache/Cellar/Logs/Temp)
- [x] Page man + complétions bash/zsh/fish
- [x] Métadonnées `Cargo.toml`, `CHANGELOG.md`, badge CI
- [x] **Publication crates.io** + tag `v0.2.0` + GitHub Release
- [x] **Formule Homebrew** préparée (`HomebrewFormula/homebrewconfig.rb`)

## 🔜 À venir

- [ ] Pousser le tap Homebrew sur un repo `homebrew-tap` (`brew install vincentlauriat/tap/homebrewconfig`)
- [ ] Détection multi-source : lire les exports `HOMEBREW_*` déjà présents hors de notre bloc
- [ ] Support **Windows** (si pertinent) ou message clair de non-support
- [ ] **Thèmes** de couleurs configurables

## 🐛 Dette technique / à surveiller

- [ ] `cursor_pos` dans la popup d'édition ne gère pas le scroll horizontal si la valeur dépasse la largeur du champ
- [ ] Pas de gestion d'erreur si `$SHELL` est inhabituel (fallback silencieux sur bash)
- [x] ~~`detect_shell_profile` suppose `~/.zprofile`~~ → résolu (auto-détection + `--profile` + cycle `p`)
