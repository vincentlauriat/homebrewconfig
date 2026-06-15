# Changes

Dated, grouped log of real changes to the repository (working journal). For the
released, versioned changelog see [`CHANGELOG.md`](CHANGELOG.md).

## 2026-06-15

### Added
- Switchable colour themes (`brew`, `midnight`, `forest`, `rose`, `mono`): cycle
  with `t`, `--theme <name>` / `--list-themes`, persisted to `config.toml`
  (`theme.rs`, `appconfig.rs`).

### Changed
- `ui.rs` now sources every colour from the active theme; hardcoded colour
  constants removed.

### Docs
- Adopted the full doc structure mandated by the parent `CLAUDE.md`:
  `ARCHITECTURE_EN.md` (English source of truth) + `ARCHITECTURE.md` (French
  mirror), `MEMORY.md`, `CHANGES.md`, `COMMANDS.md`.
- README/CHANGELOG/TODOS/ARCHITECTURE updated for themes.

### Decisions
- User preferences (theme) stored separately from managed Homebrew variables.

## 2026-06-14

### Added
- Milestone 1 — robustness: `<profile>.bak` backup, no-op apply, confirmation +
  highlighted preview, GitHub Actions CI, unit-test suite.
- Milestone 2 — ergonomics: incremental filter (`/`), configurable profile
  target (`--profile`, `p`, auto-detection), Homebrew defaults in the detail
  pane, unsaved-changes counter.
- Milestone 3 — scriptability: `--set`/`--unset`/`--apply`/`--dry-run`/`--list`/
  `--json`, TOML presets (`--export-preset`/`--import-preset`).
- Milestone 4 — distribution: `--brew-env`, directory path validation, man page,
  bash/zsh/fish completions, Cargo metadata, Homebrew formula.

### Changed
- Profile selection no longer always targets `~/.zprofile`; prefers the file
  holding the block, then existing, then preferred (zsh: `.zshrc` first).

### Docs
- Created README, ARCHITECTURE, PLAN, TODOS, CHANGELOG.

### Decisions
- Published 0.2.0 to crates.io; tagged `v0.2.0` with a GitHub Release; created
  the `vincentlauriat/homebrew-tap` tap.
