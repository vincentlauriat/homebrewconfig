# Changelog

All notable changes to this project are documented here. The format is based on
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project
adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Fixed
- The status bar now lists the `p` (cycle profile) and `t` (cycle theme) keys.

## [0.3.0] - 2026-06-15

### Added
- Colour themes: cycle with `t` in the TUI, or use `--theme <name>` /
  `--list-themes`. The choice is persisted to the user config directory and
  restored on the next launch. Ships `brew`, `midnight`, `forest`, `rose` and
  `mono`.

## [0.2.0] - 2026-06-14

### Added
- Incremental search/filter of settings with `/` (matches name, variable,
  category and description).
- Configurable write target: auto-detection prefers the profile that already
  holds the managed block, then the first existing candidate, then the
  preferred one; `--profile <PATH>` overrides it and `p` cycles it in the TUI.
- Confirmation popup with a syntax-highlighted preview before applying.
- Per-setting Homebrew defaults and an unsaved-changes counter in the header.
- Path existence check (`✓ path exists` / `⚠ path not found`) for the
  Cache/Cellar/Logs/Temp directory settings.
- Non-interactive CLI: `--set`, `--unset`, `--apply`, `--dry-run`, `--list`,
  `--json`, plus TOML presets via `--export-preset` / `--import-preset`.
- `--brew-env` prints Homebrew's effective `HOMEBREW_*` environment via `brew`.
- Man page (`man/homebrewconfig.1`) and bash/zsh/fish shell completions.
- GitHub Actions CI (fmt, clippy, build, test) and a unit-test suite.

### Changed
- Profile backup: the previous profile is copied to `<profile>.bak` before any
  write, and an unchanged apply is now a no-op.

## [0.1.0]

### Added
- Initial release: interactive TUI to browse and edit Homebrew environment
  variables and write an idempotent export block to the shell profile.

[Unreleased]: https://github.com/vincentlauriat/homebrewconfig/compare/v0.3.0...HEAD
[0.3.0]: https://github.com/vincentlauriat/homebrewconfig/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/vincentlauriat/homebrewconfig/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/vincentlauriat/homebrewconfig/releases/tag/v0.1.0
