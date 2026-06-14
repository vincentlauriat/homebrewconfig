# PLAN d'implémentation

Feuille de route de **homebrewconfig**. La v0.1.0 (état actuel) est fonctionnelle ; ce plan structure la suite par jalons.

## État actuel — v0.1.0 (livré)

TUI complète et opérationnelle : navigation, édition, application idempotente au profil shell, détection de shell, 22 réglages Homebrew couverts. Le build passe (`cargo build` ✅).

Voir [`ARCHITECTURE.md`](ARCHITECTURE.md) pour la structure et [`TODOS.md`](TODOS.md) pour le détail des tâches.

---

## Jalon 1 — Robustesse & confiance (v0.2.0)

**Objectif** : sécuriser l'écriture sur disque et prouver la logique par des tests.

### 1.1 Tests automatisés
- Tests unitaires de `config.rs` : `replace_block` (création, remplacement, préservation du reste), `generate_block` (inversion, omission des défauts), `escape_value`.
- Tests de `Setting::read_from_env` et de la sémantique `inverted`.
- Cible : couvrir la totalité de la logique pure (hors I/O terminal).

### 1.2 Sécurité de l'écriture
- Backup automatique du profil (`<profil>.bak`) avant la première écriture.
- Popup de **confirmation** avant `apply`, avec **aperçu du diff** (le bloc qui sera écrit).

### 1.3 Intégration continue
- Workflow GitHub Actions : `build` + `test` + `clippy -D warnings` + `fmt --check`.

**Critère de sortie** : `cargo test` vert en CI, écriture réversible, aucune perte possible du profil utilisateur.

---

## Jalon 2 — Ergonomie (v0.3.0)

**Objectif** : rendre l'outil agréable sur un grand nombre de réglages.

- Recherche / filtre incrémental (`/`).
- Affichage de la **valeur par défaut Homebrew** dans le panneau de détail.
- Indicateur global « N changements non appliqués » dans le header.
- Correction de la dette : scroll horizontal du champ d'édition, gestion `~/.zshrc` vs `~/.zprofile`.

**Critère de sortie** : navigation fluide, aucun réglage « masqué » par la taille de l'écran.

---

## Jalon 3 — Scriptabilité (v0.4.0)

**Objectif** : utilisable sans interface, pour l'automatisation.

- Mode CLI non interactif : `--set VAR=VALUE`, `--unset VAR`, `--apply`, `--dry-run`.
- Export/import de **presets** (TOML) pour partager une configuration entre machines.
- Sortie lisible machine (`--json`) de l'état courant.

**Critère de sortie** : un poste peut être configuré via un seul script, sans ouvrir la TUI.

---

## Jalon 4 — Distribution (v1.0.0)

**Objectif** : installation triviale pour l'utilisateur final.

- Synchronisation du catalogue avec `brew --env` (rester à jour avec Homebrew).
- Validation des chemins (Cache/Cellar/Logs/Temp).
- Publication sur **crates.io**.
- Formule **Homebrew** (`brew install homebrewconfig`) — bouclage méta-circulaire.
- Page man + complétions shell.

**Critère de sortie** : `brew install homebrewconfig` fonctionne, doc complète, API stable.

---

## Principes transverses

- **Ne jamais corrompre le profil utilisateur** : toute écriture est idempotente, délimitée et réversible.
- **Catalogue = source de vérité unique** (`build_settings`) : toute évolution des réglages passe par là.
- **Build/test vérifiés** à chaque changement (`cargo build`, `cargo test`, `cargo clippy`).
- **Séparation stricte** logique / rendu / I/O (cf. ARCHITECTURE.md).
