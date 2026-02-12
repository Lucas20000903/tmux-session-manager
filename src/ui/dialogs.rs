//! Modal dialog rendering

use ratatui::{
    layout::Alignment,
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

use crate::app::{App, NewSessionField, SessionAction};

use super::help::centered_rect;

pub fn render_confirm_action(frame: &mut Frame, app: &App) {
    let session = app.selected_session();
    let session_name = session.map(|s| s.name.as_str()).unwrap_or("?");
    let is_current_session = app
        .current_session
        .as_ref()
        .is_some_and(|c| c == session_name);

    match &app.pending_action {
        Some(action) => {
            let kills_session = matches!(action, SessionAction::Kill);
            let show_exit_warning = kills_session && is_current_session;

            let dialog_height = if show_exit_warning { 7 } else { 5 };
            let area = centered_rect(55, dialog_height, frame.area());

            let block = Block::default()
                .title(" Confirm ")
                .borders(Borders::ALL)

                .border_style(Style::default().fg(Color::Red));

            let mut lines = vec![Line::from(format!(
                "{} '{}'?",
                action.label(),
                session_name
            ))];

            if show_exit_warning {
                lines.push(Line::raw(""));
                lines.push(Line::styled(
                    "This is your current session - tmux will exit!",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ));
            }

            lines.push(Line::raw(""));
            lines.push(Line::from("[Y]es  [n]o"));

            let paragraph = Paragraph::new(Text::from(lines))
                .block(block)
                .alignment(Alignment::Center)
                .wrap(Wrap { trim: true });

            frame.render_widget(Clear, area);
            frame.render_widget(paragraph, area);
        }
        None => {}
    }
}

pub fn render_new_session_dialog(
    frame: &mut Frame,
    name: &str,
    path: &str,
    field: NewSessionField,
    path_suggestions: &[String],
    path_selected: Option<usize>,
    start_claude: bool,
) {
    // Calculate dialog height based on suggestions shown
    let max_visible = 5;
    let suggestions_to_show = if field == NewSessionField::Path && !path_suggestions.is_empty() {
        path_suggestions.len().min(max_visible)
    } else {
        0
    };
    let suggestion_extra = if suggestions_to_show > 0 {
        let total = path_suggestions.len();
        let start = if let Some(sel) = path_selected {
            if sel >= max_visible { sel - max_visible + 1 } else { 0 }
        } else {
            0
        };
        let end = (start + max_visible).min(total);
        let has_above = start > 0;
        let has_below = end < total;
        2 + if has_above { 1 } else { 0 } + if has_below { 1 } else { 0 }
    } else {
        0
    };
    let dialog_height = 10 + suggestions_to_show as u16 + suggestion_extra as u16;

    let area = centered_rect(60, dialog_height, frame.area());

    let block = Block::default()
        .title(" New Session ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let name_style = if field == NewSessionField::Name {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };

    let path_style = if field == NewSessionField::Path {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };

    let mut lines = Vec::new();

    // Start with field
    let start_label_style = if field == NewSessionField::StartWith {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };

    let (claude_style, shell_style) = if field == NewSessionField::StartWith {
        if start_claude {
            (
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                Style::default().fg(Color::DarkGray),
            )
        } else {
            (
                Style::default().fg(Color::DarkGray),
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            )
        }
    } else {
        if start_claude {
            (
                Style::default().fg(Color::White),
                Style::default().fg(Color::DarkGray),
            )
        } else {
            (
                Style::default().fg(Color::DarkGray),
                Style::default().fg(Color::White),
            )
        }
    };

    lines.push(Line::from(vec![
        Span::styled("Start: ", start_label_style),
        Span::styled("◀ ", if field == NewSessionField::StartWith {
            Style::default().fg(Color::DarkGray)
        } else {
            Style::default().fg(Color::DarkGray).add_modifier(Modifier::DIM)
        }),
        Span::styled(if start_claude { "[Claude]" } else { " Claude " }, claude_style),
        Span::raw("  "),
        Span::styled(if !start_claude { "[Shell]" } else { " Shell " }, shell_style),
        Span::styled(" ▶", if field == NewSessionField::StartWith {
            Style::default().fg(Color::DarkGray)
        } else {
            Style::default().fg(Color::DarkGray).add_modifier(Modifier::DIM)
        }),
    ]));

    lines.push(Line::raw(""));

    // Name field
    lines.push(Line::from(vec![
        Span::styled("Name: ", name_style),
        Span::raw(name),
        if field == NewSessionField::Name {
            Span::raw("_")
        } else {
            Span::raw("")
        },
    ]));

    lines.push(Line::raw(""));

    // Path field with ghost text
    let ghost_text = if field == NewSessionField::Path {
        crate::completion::complete_path(path).ghost_text
    } else {
        None
    };

    let mut path_spans = vec![
        Span::styled("Path: ", path_style),
        Span::styled(path, Style::default().fg(Color::Yellow)),
    ];

    if let Some(ref ghost) = ghost_text {
        path_spans.push(Span::styled(
            ghost,
            Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::DIM),
        ));
    }

    if field == NewSessionField::Path {
        path_spans.push(Span::raw("_"));
    }

    lines.push(Line::from(path_spans));

    // Show path suggestions when path field is active
    if field == NewSessionField::Path && !path_suggestions.is_empty() {
        lines.push(Line::styled(
            "      ────────────────────────────────────",
            Style::default().fg(Color::DarkGray),
        ));

        let max_visible = 5;
        let total = path_suggestions.len();

        // Calculate visible window based on selection
        let start = if let Some(sel) = path_selected {
            if sel >= max_visible {
                sel - max_visible + 1
            } else {
                0
            }
        } else {
            0
        };
        let end = (start + max_visible).min(total);

        if start > 0 {
            lines.push(Line::styled(
                format!("      ... {} more above", start),
                Style::default().fg(Color::DarkGray),
            ));
        }

        for i in start..end {
            let suggestion = &path_suggestions[i];
            let is_selected = path_selected == Some(i);
            let prefix = if is_selected { "    > " } else { "      " };
            let style = if is_selected {
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::DarkGray)
            };
            lines.push(Line::styled(format!("{}{}", prefix, suggestion), style));
        }

        if end < total {
            lines.push(Line::styled(
                format!("      ... {} more below", total - end),
                Style::default().fg(Color::DarkGray),
            ));
        }

        lines.push(Line::styled(
            "      ────────────────────────────────────",
            Style::default().fg(Color::DarkGray),
        ));
    }

    lines.push(Line::raw(""));
    lines.push(Line::styled(
        "Tab switch  ←→ toggle  ↑↓ select  Enter create  Esc cancel",
        Style::default().fg(Color::DarkGray),
    ));

    let text = Text::from(lines);
    let paragraph = Paragraph::new(text)
        .block(block)
        .wrap(Wrap { trim: false });

    frame.render_widget(Clear, area);
    frame.render_widget(paragraph, area);
}

pub fn render_rename_dialog(frame: &mut Frame, old_name: &str, new_name: &str) {
    let area = centered_rect(50, 6, frame.area());

    let block = Block::default()
        .title(format!(" Rename '{}' ", old_name))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let text = Text::from(vec![
        Line::from(vec![
            Span::raw("New name: "),
            Span::styled(new_name, Style::default().fg(Color::Yellow)),
            Span::raw("_"),
        ]),
        Line::raw(""),
        Line::styled(
            "Press Enter to confirm",
            Style::default().fg(Color::DarkGray),
        ),
    ]);

    let paragraph = Paragraph::new(text)
        .block(block)
        .wrap(Wrap { trim: true });

    frame.render_widget(Clear, area);
    frame.render_widget(paragraph, area);
}
