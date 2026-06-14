mod app;
mod config;
mod preset;
mod report;
mod ui;

use std::fs;
use std::io;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use crossterm::event::{
    self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyEventKind,
    KeyModifiers,
};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use app::{App, Mode, SettingKind};

const MESSAGE_TIMEOUT: Duration = Duration::from_secs(3);

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = match parse_args() {
        ArgAction::Run(cli) => cli,
        ArgAction::Exit(code) => std::process::exit(code),
    };

    // Non-interactive (scriptable) mode: apply/inspect without launching the UI.
    if cli.is_batch() {
        if let Err(e) = run_batch(&cli) {
            eprintln!("error: {}", e);
            std::process::exit(1);
        }
        return Ok(());
    }

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::with_profile(cli.profile);
    let result = run(&mut terminal, &mut app);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    result?;

    if app.has_unsaved_changes() {
        println!("Note: you have unsaved changes. Press 'a' next time to apply them.");
    }

    Ok(())
}

#[derive(Default)]
struct Cli {
    profile: Option<PathBuf>,
    sets: Vec<(String, String)>,
    unsets: Vec<String>,
    import_preset: Option<PathBuf>,
    export_preset: Option<PathBuf>,
    apply: bool,
    dry_run: bool,
    list: bool,
    json: bool,
}

impl Cli {
    /// True when any non-interactive flag is present, so we skip the TUI.
    fn is_batch(&self) -> bool {
        !self.sets.is_empty()
            || !self.unsets.is_empty()
            || self.import_preset.is_some()
            || self.export_preset.is_some()
            || self.apply
            || self.dry_run
            || self.list
            || self.json
    }
}

enum ArgAction {
    Run(Cli),
    Exit(i32),
}

fn parse_kv(s: &str) -> Option<(String, String)> {
    s.split_once('=')
        .filter(|(k, _)| !k.is_empty())
        .map(|(k, v)| (k.to_string(), v.to_string()))
}

fn parse_args() -> ArgAction {
    let mut cli = Cli::default();
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-h" | "--help" => {
                print_help();
                return ArgAction::Exit(0);
            }
            "-V" | "--version" => {
                println!("homebrewconfig {}", env!("CARGO_PKG_VERSION"));
                return ArgAction::Exit(0);
            }
            "-p" | "--profile" => match args.next() {
                Some(path) => cli.profile = Some(PathBuf::from(path)),
                None => return arg_error("--profile requires a path argument"),
            },
            "--set" => match args.next() {
                Some(kv) => match parse_kv(&kv) {
                    Some(pair) => cli.sets.push(pair),
                    None => return arg_error(&format!("--set expects VAR=VALUE, got '{}'", kv)),
                },
                None => return arg_error("--set requires VAR=VALUE"),
            },
            "--unset" => match args.next() {
                Some(var) => cli.unsets.push(var),
                None => return arg_error("--unset requires a variable name"),
            },
            "--import-preset" => match args.next() {
                Some(path) => cli.import_preset = Some(PathBuf::from(path)),
                None => return arg_error("--import-preset requires a path"),
            },
            "--export-preset" => match args.next() {
                Some(path) => cli.export_preset = Some(PathBuf::from(path)),
                None => return arg_error("--export-preset requires a path"),
            },
            "--apply" => cli.apply = true,
            "--dry-run" => cli.dry_run = true,
            "--list" => cli.list = true,
            "--json" => cli.json = true,
            other => {
                if let Some(rest) = other.strip_prefix("--profile=") {
                    cli.profile = Some(PathBuf::from(rest));
                } else if let Some(rest) = other.strip_prefix("--set=") {
                    match parse_kv(rest) {
                        Some(pair) => cli.sets.push(pair),
                        None => {
                            return arg_error(&format!("--set expects VAR=VALUE, got '{}'", rest))
                        }
                    }
                } else if let Some(rest) = other.strip_prefix("--unset=") {
                    cli.unsets.push(rest.to_string());
                } else if let Some(rest) = other.strip_prefix("--import-preset=") {
                    cli.import_preset = Some(PathBuf::from(rest));
                } else if let Some(rest) = other.strip_prefix("--export-preset=") {
                    cli.export_preset = Some(PathBuf::from(rest));
                } else {
                    return arg_error(&format!("unknown argument '{}'. Try --help.", other));
                }
            }
        }
    }
    ArgAction::Run(cli)
}

fn arg_error(msg: &str) -> ArgAction {
    eprintln!("error: {}", msg);
    ArgAction::Exit(2)
}

/// Run the requested non-interactive actions against a freshly-built app.
fn run_batch(cli: &Cli) -> Result<(), String> {
    let mut app = App::with_profile(cli.profile.clone());

    // A preset is the baseline; explicit --set/--unset override it afterwards.
    if let Some(path) = &cli.import_preset {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("failed to read {}: {}", path.display(), e))?;
        for (var, value) in preset::parse_preset(&content)? {
            if let Err(e) = app.set_var(&var, Some(value)) {
                eprintln!("warning: skipping {} from preset: {}", var, e);
            }
        }
    }

    for (var, value) in &cli.sets {
        app.set_var(var, Some(value.clone()))?;
    }
    for var in &cli.unsets {
        app.set_var(var, None)?;
    }

    if let Some(path) = &cli.export_preset {
        let toml = preset::export_preset(&app)?;
        fs::write(path, toml).map_err(|e| format!("failed to write {}: {}", path.display(), e))?;
        println!("Wrote preset to {}", path.display());
    }

    if cli.json {
        println!("{}", report::state_json(&app)?);
    }

    if cli.list {
        for s in &app.settings {
            println!("{:<28} {:<34} {}", s.name, s.env_var, s.value_display());
        }
    }

    if cli.dry_run {
        println!("{}", config::preview_block(&app));
        return Ok(());
    }

    if cli.apply {
        config::apply_config(&app)?;
        println!("Applied to {}", app.shell_profile.display());
    }

    Ok(())
}

fn print_help() {
    println!(
        "homebrewconfig {}\n\
         A TUI for configuring Homebrew environment variables.\n\n\
         USAGE:\n    \
         homebrewconfig [OPTIONS]\n\n\
         OPTIONS:\n    \
         -p, --profile <PATH>   Write to this shell profile instead of the auto-detected one\n    \
             --set VAR=VALUE    Set a Homebrew variable (repeatable; non-interactive)\n    \
             --unset VAR        Reset a variable to its Homebrew default (repeatable)\n    \
             --import-preset <PATH>  Load settings from a TOML preset (baseline)\n    \
             --export-preset <PATH>  Write current non-default settings to a TOML preset\n    \
             --apply            Write the profile without opening the UI\n    \
             --dry-run          Print the export block that would be written, then exit\n    \
             --list             Print all settings and their current values, then exit\n    \
             --json             Print the full state as JSON, then exit\n    \
         -h, --help             Print this help\n    \
         -V, --version          Print version\n\n\
         With no batch flag, the interactive TUI launches. Inside it, press 'p'\n\
         to cycle the write target and '?' for all keys.\n\n\
         EXAMPLES:\n    \
         homebrewconfig --set HOMEBREW_NO_ANALYTICS=1 --apply\n    \
         homebrewconfig --unset HOMEBREW_CLEANUP_MAX_AGE_DAYS --dry-run",
        env!("CARGO_PKG_VERSION")
    );
}

fn run<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut message_set_at: Option<Instant> = None;

    loop {
        if app.message.is_some() && message_set_at.is_none() {
            message_set_at = Some(Instant::now());
        }
        if let Some(t) = message_set_at {
            if t.elapsed() >= MESSAGE_TIMEOUT {
                app.message = None;
                message_set_at = None;
            }
        }

        terminal.draw(|f| ui::render(f, app))?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }
                let had_message = app.message.is_some();
                match app.mode {
                    Mode::Normal => {
                        if handle_normal(app, key) {
                            return Ok(());
                        }
                    }
                    Mode::Editing => handle_editing(app, key),
                    Mode::Confirming => handle_confirm(app, key),
                    Mode::Filtering => handle_filter(app, key),
                }
                if app.message.is_some() && !had_message {
                    message_set_at = Some(Instant::now());
                } else if app.message.is_none() {
                    message_set_at = None;
                }
            }
        }
    }
}

fn handle_normal(app: &mut App, key: KeyEvent) -> bool {
    match key.code {
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => return true,
        KeyCode::Char('q') => return true,
        KeyCode::Esc => {
            if app.show_help {
                app.show_help = false;
            } else {
                return true;
            }
        }
        KeyCode::Char('?') => app.show_help = !app.show_help,
        KeyCode::Up | KeyCode::Char('k') => app.select_prev(),
        KeyCode::Down | KeyCode::Char('j') => app.select_next(),
        KeyCode::Char(' ') => app.toggle_current(),
        KeyCode::Enter => app.start_editing(),
        KeyCode::Char('r') => app.reset(),
        KeyCode::Char('/') => app.start_filter(),
        KeyCode::Char('p') => app.cycle_profile(),
        KeyCode::Char('a') => app.mode = Mode::Confirming,
        _ => {}
    }
    false
}

fn handle_filter(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Enter => app.confirm_filter(),
        KeyCode::Esc => app.clear_filter(),
        KeyCode::Backspace => app.filter_backspace(),
        KeyCode::Char(c) => app.filter_push(c),
        _ => {}
    }
}

fn handle_confirm(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Enter | KeyCode::Char('y') => {
            match config::apply_config(app) {
                Ok(()) => {
                    for setting in &mut app.settings {
                        setting.modified = false;
                    }
                    let path = app.shell_profile.display().to_string();
                    app.message = Some((format!("Applied to {}", path), false));
                }
                Err(e) => {
                    app.message = Some((e, true));
                }
            }
            app.mode = Mode::Normal;
        }
        KeyCode::Esc | KeyCode::Char('n') => {
            app.mode = Mode::Normal;
            app.message = Some(("Apply cancelled".to_string(), false));
        }
        _ => {}
    }
}

fn handle_editing(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Enter => app.confirm_edit(),
        KeyCode::Esc => app.cancel_edit(),
        KeyCode::Char(c) => {
            if let SettingKind::Num = app.settings[app.selected].kind {
                if !c.is_ascii_digit() {
                    return;
                }
            }
            let byte_idx = char_to_byte(&app.input, app.cursor_pos);
            app.input.insert(byte_idx, c);
            app.cursor_pos += 1;
        }
        KeyCode::Backspace => {
            if app.cursor_pos > 0 {
                let remove_at = char_to_byte(&app.input, app.cursor_pos - 1);
                app.input.remove(remove_at);
                app.cursor_pos -= 1;
            }
        }
        KeyCode::Delete => {
            let len = app.input.chars().count();
            if app.cursor_pos < len {
                let remove_at = char_to_byte(&app.input, app.cursor_pos);
                app.input.remove(remove_at);
            }
        }
        KeyCode::Left => {
            app.cursor_pos = app.cursor_pos.saturating_sub(1);
        }
        KeyCode::Right => {
            let len = app.input.chars().count();
            if app.cursor_pos < len {
                app.cursor_pos += 1;
            }
        }
        KeyCode::Home => app.cursor_pos = 0,
        KeyCode::End => app.cursor_pos = app.input.chars().count(),
        _ => {}
    }
}

fn char_to_byte(s: &str, char_idx: usize) -> usize {
    s.char_indices()
        .nth(char_idx)
        .map(|(i, _)| i)
        .unwrap_or(s.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_kv_splits_on_first_equals() {
        assert_eq!(
            parse_kv("VAR=val"),
            Some(("VAR".to_string(), "val".to_string()))
        );
        // Value may itself contain '='.
        assert_eq!(
            parse_kv("VAR=a=b"),
            Some(("VAR".to_string(), "a=b".to_string()))
        );
        // Empty value is allowed (e.g. clearing a string).
        assert_eq!(parse_kv("VAR="), Some(("VAR".to_string(), String::new())));
    }

    #[test]
    fn parse_kv_rejects_missing_key_or_equals() {
        assert_eq!(parse_kv("noequals"), None);
        assert_eq!(parse_kv("=val"), None);
    }

    #[test]
    fn is_batch_only_when_a_batch_flag_is_set() {
        let mut cli = Cli::default();
        assert!(!cli.is_batch());
        cli.profile = Some(PathBuf::from("/x"));
        assert!(!cli.is_batch()); // profile alone still launches the TUI
        cli.apply = true;
        assert!(cli.is_batch());
    }
}
