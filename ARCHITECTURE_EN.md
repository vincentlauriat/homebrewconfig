# Architecture

> Source of truth (English). [`ARCHITECTURE.md`](ARCHITECTURE.md) is the French
> mirror and must be edited in the same change.

This document describes the internal architecture of **homebrewconfig**, written
in Rust. The tool has two faces sharing the same core:

- an interactive **TUI** (ratatui/crossterm) to browse and edit settings;
- a non-interactive **CLI** (batch mode) for scripting.

Both edit Homebrew environment variables in the user's shell profile.

## Overview

```
                          ┌──────────────────────────────┐
                          │            main.rs           │
                          │  parse_args → Cli            │
                          │  is_batch ? run_batch : TUI  │
                          └───────┬───────────────┬──────┘
              batch (CLI)         │               │  interactive (TUI)
        ┌───────────────┬─────────┘               └───────┬───────────────┐
        ▼               ▼                                 ▼               ▼
  ┌───────────┐   ┌───────────┐                     ┌───────────┐   ┌───────────┐
  │ preset.rs │   │ report.rs │                     │  event    │   │   ui.rs   │
  │ TOML I/O  │   │ JSON out  │                     │  loop     │──▶│  render   │
  └─────┬─────┘   └─────┬─────┘                     └─────┬─────┘   │ (read)    │
        │               │      ┌───────────┐              │         └───────────┘
        │               │      │  brew.rs  │              │
        │               │      │ brew env  │              │
        └───────┬───────┴──────┴─────┬─────┘              │
                ▼                    ▼                    ▼
            ┌──────────────────────────────────────────────┐
            │                   app.rs                      │
            │   App, Setting, Mode, SettingKind — state &   │
            │   domain logic (catalogue, filter, profile)   │
            └───────────────────────┬──────────────────────┘
                                    │ &App
                                    ▼
                            ┌───────────────┐
                            │   config.rs   │
                            │  shell profile│
                            │  I/O (write)  │
                            └───────────────┘
```

The app follows a model close to **Model–View–Update**:

- **Model** → `App` and `Setting` (`app.rs`) hold all state.
- **View** → `ui.rs` is a render function that only reads `&App`.
- **Update** → `main.rs` captures key events and mutates `App`.
- **Side effects** → `config.rs` isolates profile writes; `brew.rs` the only
  subprocess; `preset.rs` / `report.rs` serialization; `appconfig.rs` user prefs.

No circular dependencies: every module depends on `app.rs`, never the reverse.
`app.rs` only depends on `config.rs` (for block detection), `theme.rs` and
`appconfig.rs`.

## Modules

### `main.rs` — entry point, CLI & event loop

- **`parse_args` → `Cli`**: parses arguments into a `Cli` struct (profile,
  `sets`/`unsets`, presets, `theme`, and flags `apply`/`dry_run`/`list`/`json`/
  `brew_env`/`list_themes`). `Cli::is_batch()` decides the mode.
- **Batch mode** (`run_batch`): if any non-interactive flag is present, the TUI
  is skipped. Order: `--list-themes` / `--brew-env` (standalone) → import preset
  (baseline) → `--set`/`--unset` (override) → export preset → `--list` →
  `--json` → `--dry-run` → `--apply`. Errors exit non-zero.
- **TUI mode**:
  - **Terminal setup/teardown**: raw mode + alternate screen; restoration is
    guaranteed **before** any error is propagated.
  - **`run` loop**: status message expiry (`MESSAGE_TIMEOUT` = 3s),
    `terminal.draw`, `event::poll(100ms)`, `KeyEventKind::Press` filter.
  - **Key routing** per mode: `handle_normal`, `handle_editing`,
    `handle_confirm`, `handle_filter`. Char-aware edit cursor (`char_to_byte`).

### `app.rs` — state & domain logic

Key types:

- **`SettingKind`**: `Bool { inverted }`, `Str`, `Num`. The `inverted` flag is
  central: many Homebrew variables are **negations** (`HOMEBREW_NO_ANALYTICS`).
  The UI shows the *feature* ("Analytics ON") while the variable encodes the
  opposite; `inverted: true` translates both ways.
- **`Setting`**: static metadata + current values + `modified` + `default_hint`
  (Homebrew default shown) + `is_path` (path to validate). Chainable builders
  (`with_default`, `with_path_flag`) only enrich the relevant entries.
- **`Mode`**: `Normal`, `Editing`, `Confirming`, `Filtering`.
- **`App`**: `Vec<Setting>`, selected index, ratatui `ListState`, mode, edit
  buffer, current **filter**, **shell profile** + **candidates**, optional
  message, `show_help`, `theme_index`.

Notable logic:

- **`apply_env_value`** (pure): converts a raw `Option<String>` to the typed
  value; split from `read_from_env` to be testable without touching the process
  environment. Applies inversion.
- **`build_settings`**: catalogue of the **22 settings** (single source of truth).
- **`set_var`**: applies a value by environment variable — used by the batch CLI.
- **Filter**: `visible()` returns indices matching the filter; navigation and
  `sync_list_state` operate on that visible set.
- **Profile**: `profile_candidates()` lists candidate files per shell;
  `pick_profile(candidates, has_block, exists)` (injected predicates, testable)
  picks the file already holding the block, else the first existing, else the
  preferred; `cycle_profile` cycles the target in the TUI.
- **`path_status(exists)`** (injected predicate): existence of a path setting.
- **Theme**: `theme()` returns the active palette; `cycle_theme()` switches and
  persists; the chosen theme is loaded from `appconfig` at startup.

### `ui.rs` — rendering (read-only)

- `render` splits the screen into **header** (4) / **body** (min) / **status
  bar** (3). The body is split: settings list (55%) + detail (45%).
- Popups via `Clear` + `centered_rect`: edit, **confirmation+preview**, help.
- The header shows the version (`CARGO_PKG_VERSION`), target profile and an
  **"● N unsaved"** counter. The detail shows the Homebrew default and path
  existence status.
- **Every colour comes from the active `Theme`** (no hardcoded constants), so the
  look is swappable at runtime.
- **Invariant**: never mutates domain state (`&mut App` only for `ListState`).

### `config.rs` — shell profile persistence

Only module that writes the profile. Guarantees:

- **Delimited block** `# homebrewconfig BEGIN/END`, **idempotent**
  (`replace_block` replaces in place, else appends, preserving the rest).
- **Backup**: copies to `<profile>.bak` before overwriting; an unchanged apply is
  a **no-op**.
- **`generate_block`**: exports only non-default values (inversion handled).
- **`escape_value`**: protects `\` and `"`. **`file_contains_block`**: detects
  our block (used by profile selection).

### `preset.rs` — TOML presets (serde + toml)

- **`export_preset`**: serializes non-default settings into a `[settings]` table.
- **`parse_preset`**: reads a TOML into `(env_var, raw)` pairs.
- A preset is a **baseline**; `--set`/`--unset` override it afterwards.

### `report.rs` — JSON output (serde_json)

- **`state_json`**: typed JSON array (`bool`/`number`/`string` per kind) of all
  settings, with `value`, `modified` and `default`.

### `brew.rs` — reading the Homebrew environment

- **`brew_environment`**: runs `brew environment --shell=bash` (the only
  subprocess); `None` if `brew` is missing.
- **`parse_brew_env`** (pure): extracts `HOMEBREW_*` assignments.
- Graceful degradation: `--brew-env` mutates nothing.

### `theme.rs` — colour palettes

- **`Theme`** (`Copy`): all UI colours. Rendering sources every colour from the
  active theme, so the look is interchangeable at runtime.
- **`THEMES`**: static list (`brew`, `midnight`, `forest`, `rose`, `mono`), the
  first being the default. `index_of`/`names` for name resolution.

### `appconfig.rs` — user preferences (serde + toml)

- Persists the tool's own preferences (distinct from the managed Homebrew
  variables) in `config.toml` under `dirs::config_dir()`. Currently: the chosen
  theme.
- `load` tolerates a missing/invalid file (falls back to defaults); `save` is
  best-effort.

## Data flow

### Startup (TUI)
```
main() → parse_args() → App::with_profile(profile)
       → build_settings() → read_from_env() (×22)
       → profile_candidates() / pick_profile() → appconfig::load() → run() loop
```

### Apply (`a` → confirm → `y`)
```
handle_normal('a') → Mode::Confirming → render_confirm_popup (preview)
handle_confirm('y') → config::apply_config(&app)
   → read profile → generate_block() → replace_block()
   → backup <profile>.bak → fs::write() → clear dirty flags
```

### Batch mode (CLI)
```
parse_args() → Cli::is_batch() == true → run_batch()
   → import preset → set/unset → export preset → list/json/dry-run/apply
```

## Design decisions

- **Static catalogue in code**: zero parsing, `&'static str` metadata, the
  Homebrew variable list evolves slowly.
- **Idempotent managed block + backup**: we own only our section, the rest of the
  profile stays intact, and every write is reversible.
- **Pure, injectable logic** (`apply_env_value`, `pick_profile`, `path_status`,
  `parse_brew_env`, `generate_block(&[Setting])`): testable without I/O — hence a
  **54-test** suite touching neither terminal nor disk.
- **Shared TUI/CLI core**: the same `App`/`Setting` logic serves both modes.
- **Teardown before error propagation**: terminal robustness.

## Dependencies

| Crate | Role |
|-------|------|
| `ratatui` 0.29 | TUI widgets and layout |
| `crossterm` 0.28 | Terminal backend (raw mode, events, alternate screen) |
| `dirs` 5 | Cross-platform home/config directory resolution |
| `serde` 1 + `toml` 0.8 | TOML presets and user config |
| `serde_json` 1 | `--json` output |

## Extension points

- **Add a setting**: one entry in `App::build_settings` (optionally with
  `.with_default(...)` / `.with_path_flag()`). Rendering, env read, export,
  presets and JSON handle it automatically.
- **New setting kind**: extend `SettingKind` then complete the `match`es.
- **New shell**: a branch in `App::profile_candidates`.
- **New CLI command**: a field in `Cli`, a branch in `parse_args`, handling in
  `run_batch`.
- **New theme**: one entry in `theme::THEMES`; cycling, `--theme` and persistence
  pick it up automatically.
