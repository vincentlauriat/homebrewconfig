---
project: homebrewconfig
last_updated: 2026-06-15
---

# Project memory

State of record for homebrewconfig, kept in sync across sessions. See
[`TODOS.md`](TODOS.md) for the task list and [`CHANGES.md`](CHANGES.md) for the
dated change log.

## What it is

A Rust tool — interactive **TUI** plus scriptable **CLI** — that edits Homebrew
environment variables in the user's shell profile by writing an idempotent,
delimited, reversible export block.

## Current state

- **Version**: 0.2.0 (published).
- **Distribution**: on crates.io (`cargo install homebrewconfig`), GitHub tag
  `v0.2.0` + Release, and a Homebrew tap
  ([`vincentlauriat/homebrew-tap`](https://github.com/vincentlauriat/homebrew-tap),
  `brew install vincentlauriat/tap/homebrewconfig`).
- **Quality**: 54 unit tests, CI (fmt/clippy `-D warnings`/build/test) green.
- **Roadmap**: milestones 1–4 shipped. Post-0.2.0 work (themes) is on `main`,
  unreleased.

## Key decisions

- **Idempotent managed block + backup**: only our `# homebrewconfig BEGIN/END`
  section is owned; `<profile>.bak` written before every overwrite; unchanged
  apply is a no-op.
- **Static catalogue** (`App::build_settings`) is the single source of truth for
  the 22 settings.
- **Pure, injectable logic** for testability without I/O (env conversion, profile
  pick, path status, brew parsing, block generation).
- **Shared core** between TUI and CLI (`App`/`Setting`).
- **Profile auto-detection** prefers the file already holding the block, then the
  first existing candidate, then the preferred one (zsh: `.zshrc` before
  `.zprofile`); overridable via `--profile` and the `p` key.
- **User preferences** (e.g. theme) live in `config.toml` under the platform
  config dir, separate from the managed Homebrew variables.

## Conventions

- Communication with Vincent: French (full accents).
- Code identifiers, commits, doc files: English. `ARCHITECTURE.md` is the French
  mirror of `ARCHITECTURE_EN.md` (source of truth).
- Conventional commits; never push to `main` directly (feature branch + PR);
  confirm before releases/tags and external/public actions.

## Workflow notes

- Work proceeds milestone by milestone via feature branch → PR → merge.
- `gh pr merge --delete-branch` on a base branch of a stacked PR will close the
  child PR; merge bottom-up or avoid stacking.
