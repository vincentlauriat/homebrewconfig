use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap};
use ratatui::Frame;

use crate::app::{App, Mode, SettingKind};
use crate::config;

const BREW_GOLD: Color = Color::Rgb(255, 180, 0);
const BREW_AMBER: Color = Color::Rgb(230, 120, 0);
const ON_COLOR: Color = Color::Rgb(80, 200, 120);
const OFF_COLOR: Color = Color::Rgb(120, 120, 130);
const MODIFIED_COLOR: Color = Color::Rgb(100, 180, 255);
const CATEGORY_COLOR: Color = Color::Rgb(180, 140, 255);
const BG: Color = Color::Rgb(22, 22, 32);

pub fn render(f: &mut Frame, app: &mut App) {
    let area = f.area();
    f.render_widget(Block::default().style(Style::default().bg(BG)), area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4),
            Constraint::Min(1),
            Constraint::Length(3),
        ])
        .split(area);

    render_header(f, app, chunks[0]);
    render_body(f, app, chunks[1]);
    render_statusbar(f, app, chunks[2]);

    if app.mode == Mode::Editing {
        render_edit_popup(f, app);
    }

    if app.mode == Mode::Confirming {
        render_confirm_popup(f, app);
    }

    if app.show_help {
        render_help_popup(f);
    }
}

fn render_header(f: &mut Frame, app: &App, area: Rect) {
    let profile = app.shell_profile.display().to_string();
    let profile = profile.replace(
        &dirs::home_dir()
            .map(|h| h.display().to_string())
            .unwrap_or_default(),
        "~",
    );

    let title = Line::from(vec![
        Span::styled(
            "🍺 homebrewconfig",
            Style::default().fg(BREW_GOLD).add_modifier(Modifier::BOLD),
        ),
        Span::styled(" v0.1.0", Style::default().fg(BREW_AMBER)),
    ]);
    let mut subtitle_spans = vec![
        Span::styled("Profile: ", Style::default().fg(OFF_COLOR)),
        Span::styled(profile, Style::default().fg(MODIFIED_COLOR)),
    ];
    let unsaved = app.settings.iter().filter(|s| s.modified).count();
    if unsaved > 0 {
        subtitle_spans.push(Span::styled("   ", Style::default()));
        subtitle_spans.push(Span::styled(
            format!("● {} unsaved", unsaved),
            Style::default()
                .fg(MODIFIED_COLOR)
                .add_modifier(Modifier::BOLD),
        ));
    }
    let subtitle = Line::from(subtitle_spans);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(BREW_AMBER));
    let para = Paragraph::new(vec![title, subtitle]).block(block);
    f.render_widget(para, area);
}

fn render_body(f: &mut Frame, app: &mut App, area: Rect) {
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
        .split(area);

    render_settings_list(f, app, cols[0]);
    render_detail(f, app, cols[1]);
}

fn render_settings_list(f: &mut Frame, app: &mut App, area: Rect) {
    let visible = app.visible();
    let mut items: Vec<ListItem> = Vec::new();
    let mut current_category: Option<&str> = None;

    if visible.is_empty() {
        items.push(ListItem::new(Line::from(Span::styled(
            "  (no matches)",
            Style::default().fg(OFF_COLOR),
        ))));
    }

    for &i in &visible {
        let setting = &app.settings[i];
        if current_category != Some(setting.category) {
            if current_category.is_some() {
                items.push(ListItem::new(Line::from("")));
            }
            items.push(ListItem::new(Line::from(Span::styled(
                format!("  {}", setting.category),
                Style::default()
                    .fg(CATEGORY_COLOR)
                    .add_modifier(Modifier::BOLD),
            ))));
            current_category = Some(setting.category);
        }

        let selected = i == app.selected;
        let pointer = if selected { "❯ " } else { "  " };
        let pointer_style = if selected {
            Style::default().fg(BREW_GOLD).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(OFF_COLOR)
        };

        let name_style = if selected {
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Gray)
        };

        let mut spans = vec![
            Span::styled(pointer, pointer_style),
            Span::styled(format!("{:<18}", setting.name), name_style),
        ];

        match &setting.kind {
            SettingKind::Bool { .. } => {
                let (label, color) = if setting.is_feature_on() {
                    ("[  ON  ]", ON_COLOR)
                } else {
                    ("[ OFF  ]", OFF_COLOR)
                };
                spans.push(Span::styled(label, Style::default().fg(color)));
            }
            _ => {
                let display = setting.value_display();
                let truncated = truncate(&display, 18);
                spans.push(Span::styled(
                    format!("{:<18}", truncated),
                    Style::default().fg(BREW_AMBER),
                ));
            }
        }

        if setting.modified {
            spans.push(Span::styled(" ●", Style::default().fg(MODIFIED_COLOR)));
        }

        let line_style = if selected {
            Style::default().bg(Color::Rgb(40, 40, 55))
        } else {
            Style::default()
        };

        items.push(ListItem::new(Line::from(spans)).style(line_style));
    }

    let title = if app.filter.is_empty() {
        " Settings ".to_string()
    } else {
        format!(" Settings (filter: {}) ", app.filter)
    };
    let block = Block::default()
        .title(Span::styled(
            title,
            Style::default().fg(BREW_GOLD).add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(BREW_AMBER));

    let list = List::new(items).block(block);
    f.render_stateful_widget(list, area, &mut app.list_state);
}

fn render_detail(f: &mut Frame, app: &App, area: Rect) {
    let detail_block = || {
        Block::default()
            .title(Span::styled(
                " Detail ",
                Style::default().fg(BREW_GOLD).add_modifier(Modifier::BOLD),
            ))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(BREW_AMBER))
    };

    if !app.visible().contains(&app.selected) {
        let para = Paragraph::new(Line::from(Span::styled(
            "No setting matches the filter.",
            Style::default().fg(OFF_COLOR),
        )))
        .block(detail_block())
        .wrap(Wrap { trim: true });
        f.render_widget(para, area);
        return;
    }

    let setting = &app.settings[app.selected];

    let mut lines: Vec<Line> = vec![
        Line::from(Span::styled(
            setting.name,
            Style::default().fg(BREW_GOLD).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
    ];
    lines.push(Line::from(Span::styled(
        setting.description,
        Style::default().fg(Color::Gray),
    )));
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "Env var:",
        Style::default().fg(OFF_COLOR),
    )));
    lines.push(Line::from(Span::styled(
        setting.env_var,
        Style::default().fg(MODIFIED_COLOR),
    )));
    lines.push(Line::from(""));

    match &setting.kind {
        SettingKind::Bool { .. } => {
            let (dot, label, color) = if setting.is_feature_on() {
                ("●", "Enabled", ON_COLOR)
            } else {
                ("●", "Disabled", OFF_COLOR)
            };
            lines.push(Line::from(vec![
                Span::styled("Status: ", Style::default().fg(OFF_COLOR)),
                Span::styled(format!("{} {}", dot, label), Style::default().fg(color)),
            ]));
        }
        _ => {
            lines.push(Line::from(vec![
                Span::styled("Value: ", Style::default().fg(OFF_COLOR)),
                Span::styled(setting.value_display(), Style::default().fg(BREW_AMBER)),
            ]));
        }
    }

    if !setting.default_hint.is_empty() {
        lines.push(Line::from(vec![
            Span::styled("Homebrew default: ", Style::default().fg(OFF_COLOR)),
            Span::styled(setting.default_hint, Style::default().fg(Color::Gray)),
        ]));
    }

    if let Some(exists) = setting.path_status(|p| std::path::Path::new(p).exists()) {
        let (mark, label, color) = if exists {
            ("✓", "path exists", ON_COLOR)
        } else {
            ("⚠", "path not found", Color::Rgb(230, 160, 60))
        };
        lines.push(Line::from(vec![
            Span::styled("Path: ", Style::default().fg(OFF_COLOR)),
            Span::styled(format!("{} {}", mark, label), Style::default().fg(color)),
        ]));
    }

    if setting.modified {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "● modified (unsaved)",
            Style::default().fg(MODIFIED_COLOR),
        )));
    }

    let para = Paragraph::new(lines)
        .block(detail_block())
        .wrap(Wrap { trim: true });
    f.render_widget(para, area);
}

fn render_statusbar(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(BREW_AMBER));

    let key = |k: &'static str| {
        Span::styled(
            k,
            Style::default().fg(BREW_GOLD).add_modifier(Modifier::BOLD),
        )
    };
    let txt = |t: &'static str| Span::styled(t, Style::default().fg(Color::Gray));

    let line = if app.mode == Mode::Filtering {
        Line::from(vec![
            Span::styled(
                "Filter: ",
                Style::default().fg(BREW_GOLD).add_modifier(Modifier::BOLD),
            ),
            Span::styled(app.filter.clone(), Style::default().fg(Color::White)),
            Span::styled("▌", Style::default().fg(BREW_GOLD)),
            txt("   "),
            key("[Enter]"),
            txt(" apply  "),
            key("[Esc]"),
            txt(" clear"),
        ])
    } else if let Some((msg, is_error)) = &app.message {
        let color = if *is_error {
            Color::Rgb(230, 90, 90)
        } else {
            ON_COLOR
        };
        Line::from(Span::styled(
            msg.clone(),
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        ))
    } else {
        Line::from(vec![
            key("[↑↓]"),
            txt(" nav  "),
            key("[Space]"),
            txt(" toggle  "),
            key("[Enter]"),
            txt(" edit  "),
            key("[/]"),
            txt(" filter  "),
            key("[a]"),
            txt(" apply  "),
            key("[r]"),
            txt(" reset  "),
            key("[?]"),
            txt(" help  "),
            key("[q]"),
            txt(" quit"),
        ])
    };

    let para = Paragraph::new(line).block(block).alignment(Alignment::Left);
    f.render_widget(para, area);
}

fn render_edit_popup(f: &mut Frame, app: &App) {
    let setting = &app.settings[app.selected];
    let area = centered_rect(60, 40, f.area());
    f.render_widget(Clear, area);

    let block = Block::default()
        .title(Span::styled(
            format!(" Edit: {} ", setting.name),
            Style::default().fg(BREW_GOLD).add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(BREW_GOLD))
        .style(Style::default().bg(BG));
    f.render_widget(block, area);

    let inner = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(2),
            Constraint::Length(3),
            Constraint::Length(2),
        ])
        .split(area);

    let desc = Paragraph::new(setting.description)
        .style(Style::default().fg(Color::Gray))
        .wrap(Wrap { trim: true });
    f.render_widget(desc, inner[0]);

    let input_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(BREW_AMBER));
    let input = Paragraph::new(app.input.as_str())
        .style(Style::default().fg(Color::White))
        .block(input_block);
    f.render_widget(input, inner[1]);

    let cursor_x = inner[1].x + 1 + app.cursor_pos as u16;
    let cursor_y = inner[1].y + 1;
    f.set_cursor_position((cursor_x, cursor_y));

    let hint = match &setting.kind {
        SettingKind::Num => "Enter a number   [Enter] confirm  [Esc] cancel",
        _ => "Leave empty for default   [Enter] confirm  [Esc] cancel",
    };
    let hint_para = Paragraph::new(hint).style(Style::default().fg(OFF_COLOR));
    f.render_widget(hint_para, inner[2]);
}

fn render_confirm_popup(f: &mut Frame, app: &App) {
    let area = centered_rect(70, 70, f.area());
    f.render_widget(Clear, area);

    let profile = app.shell_profile.display().to_string().replace(
        &dirs::home_dir()
            .map(|h| h.display().to_string())
            .unwrap_or_default(),
        "~",
    );

    let block = Block::default()
        .title(Span::styled(
            " Apply changes? ",
            Style::default().fg(BREW_GOLD).add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(BREW_GOLD))
        .style(Style::default().bg(BG));
    f.render_widget(block, area);

    let inner = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(2),
            Constraint::Min(1),
            Constraint::Length(2),
        ])
        .split(area);

    let target = Paragraph::new(Line::from(vec![
        Span::styled("Will write to ", Style::default().fg(Color::Gray)),
        Span::styled(profile, Style::default().fg(MODIFIED_COLOR)),
    ]))
    .wrap(Wrap { trim: true });
    f.render_widget(target, inner[0]);

    let preview = config::preview_block(app);
    let mut lines: Vec<Line> = Vec::new();
    for raw in preview.lines() {
        let style = if raw.starts_with("# homebrewconfig") {
            Style::default().fg(OFF_COLOR)
        } else if raw.starts_with('#') {
            Style::default()
                .fg(CATEGORY_COLOR)
                .add_modifier(Modifier::BOLD)
        } else if raw.starts_with("export") {
            Style::default().fg(ON_COLOR)
        } else {
            Style::default().fg(Color::Gray)
        };
        lines.push(Line::from(Span::styled(raw.to_string(), style)));
    }

    let body_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(BREW_AMBER));
    let body = Paragraph::new(lines)
        .block(body_block)
        .wrap(Wrap { trim: false });
    f.render_widget(body, inner[1]);

    let hint = Line::from(vec![
        Span::styled(
            "[y/Enter]",
            Style::default().fg(BREW_GOLD).add_modifier(Modifier::BOLD),
        ),
        Span::styled(" apply   ", Style::default().fg(Color::Gray)),
        Span::styled(
            "[n/Esc]",
            Style::default().fg(BREW_GOLD).add_modifier(Modifier::BOLD),
        ),
        Span::styled(" cancel", Style::default().fg(Color::Gray)),
    ]);
    let hint_para = Paragraph::new(hint);
    f.render_widget(hint_para, inner[2]);
}

fn render_help_popup(f: &mut Frame) {
    let area = centered_rect(50, 70, f.area());
    f.render_widget(Clear, area);

    let key = |k: &'static str, d: &'static str| {
        Line::from(vec![
            Span::styled(
                format!("  {:<12}", k),
                Style::default().fg(BREW_GOLD).add_modifier(Modifier::BOLD),
            ),
            Span::styled(d, Style::default().fg(Color::Gray)),
        ])
    };

    let lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            "  Navigation",
            Style::default()
                .fg(CATEGORY_COLOR)
                .add_modifier(Modifier::BOLD),
        )),
        key("↑ / k", "Move selection up"),
        key("↓ / j", "Move selection down"),
        key("/", "Filter settings by name/var"),
        Line::from(""),
        Line::from(Span::styled(
            "  Editing",
            Style::default()
                .fg(CATEGORY_COLOR)
                .add_modifier(Modifier::BOLD),
        )),
        key("Space", "Toggle a boolean setting"),
        key("Enter", "Edit a string/number setting"),
        key("Esc", "Cancel editing / close help"),
        Line::from(""),
        Line::from(Span::styled(
            "  Actions",
            Style::default()
                .fg(CATEGORY_COLOR)
                .add_modifier(Modifier::BOLD),
        )),
        key("a", "Apply (asks to confirm + preview)"),
        key("r", "Reset to current environment"),
        key("p", "Cycle the target shell profile"),
        key("?", "Toggle this help"),
        key("q", "Quit"),
        Line::from(""),
        Line::from(Span::styled(
            "  Press ? or Esc to close",
            Style::default().fg(OFF_COLOR),
        )),
    ];

    let block = Block::default()
        .title(Span::styled(
            " Help ",
            Style::default().fg(BREW_GOLD).add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(BREW_GOLD))
        .style(Style::default().bg(BG));
    let para = Paragraph::new(lines).block(block);
    f.render_widget(para, area);
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(vertical[1])[1]
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let mut result: String = s.chars().take(max.saturating_sub(1)).collect();
        result.push('…');
        result
    }
}
