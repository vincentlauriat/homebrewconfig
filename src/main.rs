mod app;
mod config;
mod ui;

use std::io;
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
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();
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
        KeyCode::Char('a') => match config::apply_config(app) {
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
        },
        _ => {}
    }
    false
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
