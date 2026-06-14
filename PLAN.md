# PLAN d'implémentation

Feuille de route de **homebrewconfig**. Les jalons 1 à 4 sont **livrés** et
publiés dans la **v0.2.0** (sur [crates.io](https://crates.io/crates/homebrewconfig)).
Ce document garde l'historique de la feuille de route et liste la suite.

## État actuel — v0.2.0 (publiée)

TUI **et** CLI scriptable. 22 réglages Homebrew couverts, écriture idempotente et
réversible, **47 tests**, CI verte. Publiée sur crates.io et taguée `v0.2.0`.

Voir [`ARCHITECTURE.md`](ARCHITECTURE.md) pour la structure et
[`TODOS.md`](TODOS.md) pour le détail des tâches.

---

## Jalon 1 — Robustesse & confiance ✅

- Tests unitaires de la logique pure (`config.rs`, `apply_env_value`).
- Backup `<profil>.bak` avant écriture ; no-op si rien ne change.
- Popup de **confirmation** avec **aperçu** du bloc avant `apply`.
- CI GitHub Actions : `fmt --check` + `clippy -D warnings` + `build` + `test`.

## Jalon 2 — Ergonomie ✅

- Recherche / filtre incrémental (`/`).
- Valeur par défaut Homebrew dans le détail + compteur « N unsaved » dans le header.
- Cible de profil configurable : auto-détection (`.zshrc` préféré, fichier
  contenant déjà le bloc), `--profile`, cycle `p` — résout la dette `.zprofile`
  vs `.zshrc`.

## Jalon 3 — Scriptabilité ✅

- CLI non interactive : `--set`, `--unset`, `--apply`, `--dry-run`, `--list`.
- Export/import de **presets TOML** (`--export-preset` / `--import-preset`).
- Sortie **`--json`** de l'état courant.

## Jalon 4 — Distribution ✅ (local) / 🟡 (externe)

- ✅ `--brew-env` : lecture de l'environnement effectif via `brew`.
- ✅ Validation d'existence des chemins (Cache/Cellar/Logs/Temp).
- ✅ Page **man** + complétions **bash/zsh/fish**.
- ✅ Métadonnées `Cargo.toml`, `CHANGELOG.md`, badge CI.
- ✅ **Publication crates.io** + tag `v0.2.0` + GitHub Release.
- 🟡 **Formule Homebrew** (tap perso) — préparée dans
  [`HomebrewFormula/`](HomebrewFormula/), à pousser sur un repo de tap.

---

## Suite envisagée (post-0.2.0)

| Idée | Note |
|------|------|
| Tap Homebrew publié (`brew install vincentlauriat/tap/homebrewconfig`) | dépend d'un repo `homebrew-tap` |
| Scroll horizontal du champ d'édition (dette TUI) | confort sur valeurs longues |
| Thèmes de couleurs configurables | personnalisation |
| Support Windows ou message clair de non-support | portée |
| Détection multi-source (lire les exports déjà présents hors de notre bloc) | robustesse |

---

## Principes transverses

- **Ne jamais corrompre le profil utilisateur** : écriture idempotente, délimitée,
  réversible (backup).
- **Catalogue = source de vérité unique** (`build_settings`).
- **Logique pure et testée** : prédicats/valeurs injectables, 47 tests sans I/O.
- **Build/test/clippy vérifiés** à chaque changement, en local et en CI.
- **Séparation stricte** logique / rendu / I/O (cf. ARCHITECTURE.md).
