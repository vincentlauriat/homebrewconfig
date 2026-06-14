# homebrewconfig

A terminal UI for configuring [Homebrew](https://brew.sh) environment variables — no manual shell profile editing required.

```
╔══════════════════════════════════════════════════════════════╗
║              🍺 homebrewconfig v0.1.0                        ║
║              Profile: ~/.zprofile                            ║
╚══════════════════════════════════════════════════════════════╝
┌─────────────────────────────────┐┌────────────────────────────┐
│ Settings                        ││ Detail                     │
│  Privacy                        ││ Analytics                  │
│ ❯ Analytics           [  ON  ]  ││                            │
│   Env Hints           [  ON  ]  ││ Disable sending analytics  │
│                                 ││ to Google Analytics        │
│  Updates                        ││                            │
│   Auto Update         [  ON  ]  ││ Env var:                   │
│   Install Upgrade     [  ON  ]  ││ HOMEBREW_NO_ANALYTICS      │
│   Install Cleanup     [  ON  ]  ││                            │
│   Cleanup Age         [ 120  ]  ││ Status: ● Enabled          │
│                                 ││                            │
│  Display                        ││ [Space] to toggle          │
│   Color               [  ON  ]  │└────────────────────────────┘
└─────────────────────────────────┘
┌──────────────────────────────────────────────────────────────┐
│ [↑↓] navigate  [Space] toggle  [Enter] edit  [a] apply       │
│ [r] reset  [?] help  [q] quit                                │
└──────────────────────────────────────────────────────────────┘
```

## Features

- Browse and edit all Homebrew environment variables in one place
- Filter settings instantly with `/` (matches name, variable, category and description)
- Toggle boolean settings with `Space`, edit strings and numbers inline
- See each setting's Homebrew default and an unsaved-changes counter at a glance
- Detects your shell automatically (zsh, bash, fish)
- Writes a clean, idempotent block to your shell profile — re-running never duplicates exports
- Reads current environment on launch so existing settings are reflected immediately

## Installation

### From source

```bash
git clone https://github.com/vincentlauriat/homebrewconfig
cd homebrewconfig
cargo install --path .
```

### Prerequisites

- Rust 1.70+ ([rustup.rs](https://rustup.rs))
- A terminal with 256-color support

## Usage

```bash
homebrewconfig                      # auto-detect the shell profile
homebrewconfig --profile ~/.zshrc   # write to a specific profile
```

Navigate with arrow keys or `j`/`k`, make your changes, then press `a` to apply. A confirmation popup previews the exact export block that will be written; press `y`/`Enter` to confirm or `n`/`Esc` to cancel. The tool writes the block to your shell profile, backs up the previous version to `<profile>.bak`, and leaves the rest of the file untouched.

Press `p` at any time to cycle the write target between the candidate profiles for your shell.

### Options

| Flag | Description |
|------|-------------|
| `-p`, `--profile <PATH>` | Write to this profile instead of the auto-detected one |
| `--set VAR=VALUE` | Set a Homebrew variable (repeatable; non-interactive) |
| `--unset VAR` | Reset a variable to its Homebrew default (repeatable) |
| `--import-preset <PATH>` | Load settings from a TOML preset (used as a baseline) |
| `--export-preset <PATH>` | Write current non-default settings to a TOML preset |
| `--apply` | Write the profile without opening the UI |
| `--dry-run` | Print the export block that would be written, then exit |
| `--list` | Print all settings and their current values, then exit |
| `--json` | Print the full state as JSON (for `jq` etc.), then exit |
| `-h`, `--help` | Print help |
| `-V`, `--version` | Print version |

### Non-interactive use

Any of `--set`, `--unset`, `--apply`, `--dry-run` or `--list` runs without the
TUI, so the tool is scriptable:

```bash
# Preview what would be written, without touching anything
homebrewconfig --set HOMEBREW_NO_ANALYTICS=1 --dry-run

# Apply changes straight to the profile (backs up to <profile>.bak)
homebrewconfig --set HOMEBREW_NO_ANALYTICS=1 \
               --set HOMEBREW_CLEANUP_MAX_AGE_DAYS=30 --apply

# Reset a variable to its default and inspect the result
homebrewconfig --unset HOMEBREW_CLEANUP_MAX_AGE_DAYS --list

# Machine-readable state, e.g. which settings differ from their default
homebrewconfig --json | jq '.[] | select(.modified)'
```

#### Presets

Capture your current configuration as a shareable TOML file and re-apply it on
another machine:

```bash
# Save the current non-default settings
homebrewconfig --export-preset team-defaults.toml

# Apply a preset somewhere else (and write it to the profile)
homebrewconfig --import-preset team-defaults.toml --apply
```

A preset is the baseline: any `--set` / `--unset` on the same command line
overrides it. Unknown variables in a preset are skipped with a warning, so
presets from a newer version stay forward-compatible. The format is a flat
`[settings]` table of `HOMEBREW_* = "value"` entries.

`VAR` is the Homebrew environment variable name. For boolean `HOMEBREW_NO_*`
variables, `--set` makes the variable present (feature off) and `--unset`
removes it (feature on), mirroring how Homebrew reads them.

## Keybindings

| Key | Action |
|-----|--------|
| `↑` / `↓` or `k` / `j` | Move selection |
| `/` | Filter settings by name, variable, category or description |
| `Space` | Toggle a boolean setting |
| `Enter` | Edit a string or number setting |
| `a` | Apply all changes (shows a confirmation + preview first) |
| `r` | Reset to current environment values |
| `p` | Cycle the target shell profile |
| `?` | Toggle help |
| `Esc` | Close help / quit |
| `q` | Quit |
| `Ctrl+C` | Quit |

## Settings

| Category | Setting | Environment Variable |
|----------|---------|----------------------|
| Privacy | Analytics | `HOMEBREW_NO_ANALYTICS` |
| Privacy | Env Hints | `HOMEBREW_NO_ENV_HINTS` |
| Updates | Auto Update | `HOMEBREW_NO_AUTO_UPDATE` |
| Updates | Install Upgrade | `HOMEBREW_NO_INSTALL_UPGRADE` |
| Updates | Install Cleanup | `HOMEBREW_NO_INSTALL_CLEANUP` |
| Updates | Cleanup Age | `HOMEBREW_CLEANUP_MAX_AGE_DAYS` |
| Display | Color | `HOMEBREW_NO_COLOR` |
| Display | Emoji | `HOMEBREW_NO_EMOJI` |
| Display | Verbose | `HOMEBREW_VERBOSE` |
| Display | Debug | `HOMEBREW_DEBUG` |
| Display | Install Badge | `HOMEBREW_INSTALL_BADGE` |
| Directories | Cache | `HOMEBREW_CACHE` |
| Directories | Cellar | `HOMEBREW_CELLAR` |
| Directories | Logs | `HOMEBREW_LOGS` |
| Directories | Temp | `HOMEBREW_TEMP` |
| Tools | Editor | `HOMEBREW_EDITOR` |
| Tools | Curl Retries | `HOMEBREW_CURL_RETRIES` |
| Security | Insecure Redirect | `HOMEBREW_NO_INSECURE_REDIRECT` |
| Network | Bottle Domain | `HOMEBREW_BOTTLE_DOMAIN` |
| Network | Brew Remote | `HOMEBREW_BREW_GIT_REMOTE` |
| Network | Core Remote | `HOMEBREW_CORE_GIT_REMOTE` |
| Network | GitHub Token | `HOMEBREW_GITHUB_API_TOKEN` |

## How it works

When you press `a`, homebrewconfig writes a block like this to your shell profile:

```sh
# homebrewconfig BEGIN
# Generated by homebrewconfig — do not edit manually

# Privacy
export HOMEBREW_NO_ANALYTICS=1

# Updates
export HOMEBREW_CLEANUP_MAX_AGE_DAYS=30

# Network
export HOMEBREW_GITHUB_API_TOKEN="ghp_..."
# homebrewconfig END
```

Running the tool again replaces only this block, leaving everything else in your profile untouched. To remove all managed settings, delete the block between the `BEGIN` and `END` markers.

## Shell detection

The target profile is auto-detected from `$SHELL`. For each shell a list of
candidate files is considered, in preference order, and the tool picks:

1. the candidate that **already contains** a homebrewconfig block (so re-runs
   stay in the same file), otherwise
2. the first candidate that **already exists**, otherwise
3. the first (preferred) candidate.

| Shell | Candidates (preferred first) |
|-------|------------------------------|
| zsh | `~/.zshrc`, `~/.zprofile` |
| bash | `~/.bashrc`, `~/.bash_profile`, `~/.profile` |
| fish | `~/.config/fish/config.fish` |

Override the choice entirely with `--profile <PATH>`, or cycle through the
candidates live with `p` inside the TUI.

## Documentation

| Document | Contenu |
|----------|---------|
| [ARCHITECTURE.md](ARCHITECTURE.md) | Architecture interne, modules, flux de données, décisions de conception |
| [PLAN.md](PLAN.md) | Feuille de route par jalons (v0.2.0 → v1.0.0) |
| [TODOS.md](TODOS.md) | Suivi détaillé des tâches faites et à venir |

## License

MIT
