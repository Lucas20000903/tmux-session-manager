//! UI rendering for the TUI application

mod dialogs;
mod help;

use ansi_to_tui::IntoText;
use ratatui::{
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Clear, List, ListItem, Paragraph, StatefulWidget},
    Frame,
};
use unicode_width::UnicodeWidthStr;

use crate::app::{App, Mode};
use crate::session::ClaudeCodeStatus;

/// Render the application UI
pub fn render(frame: &mut Frame, app: &mut App) {
    let area = frame.area();

    // Auto-hide preview on small terminals
    let show_preview = app.show_preview && area.height >= 30;

    let status_bar_area;

    if show_preview {
        let available_height = area.height.saturating_sub(4);
        let preview_height = (available_height * 50 / 100).clamp(5, 30);

        let layout = Layout::vertical([
            Constraint::Length(1),              // Header
            Constraint::Min(3),                 // Session list
            Constraint::Length(preview_height), // Preview pane
            Constraint::Length(1),              // Status bar
            Constraint::Length(1),              // Footer
        ])
        .split(area);

        render_header(frame, app, layout[0]);
        render_session_list(frame, app, layout[1]);
        render_preview(frame, app, layout[2]);
        render_status_bar(frame, app, layout[3]);
        render_footer(frame, app, layout[4]);
        status_bar_area = layout[3];
    } else {
        let layout = Layout::vertical([
            Constraint::Length(1), // Header
            Constraint::Min(3),   // Session list
            Constraint::Length(1), // Status bar
            Constraint::Length(1), // Footer
        ])
        .split(area);

        render_header(frame, app, layout[0]);
        render_session_list(frame, app, layout[1]);
        render_status_bar(frame, app, layout[2]);
        render_footer(frame, app, layout[3]);
        status_bar_area = layout[2];
    }

    // Render modal overlays
    match &app.mode {
        Mode::ConfirmAction => {
            dialogs::render_confirm_action(frame, app);
        }
        Mode::NewSession {
            name,
            path,
            field,
            path_suggestions,
            path_selected,
            start_claude,
        } => {
            dialogs::render_new_session_dialog(
                frame,
                name,
                path,
                *field,
                path_suggestions,
                *path_selected,
                *start_claude,
            );
        }
        Mode::Rename { old_name, new_name } => {
            dialogs::render_rename_dialog(frame, old_name, new_name);
        }
        Mode::Filter { input } => {
            render_filter_bar(frame, input, status_bar_area);
        }
        Mode::Help => {
            help::render_help(frame);
        }
        Mode::Normal | Mode::ActionMenu => {}
    }

    // Render error/message overlay
    if let Some(ref error) = app.error {
        help::render_message(frame, error, Color::Red);
    } else if let Some(ref message) = app.message {
        help::render_message(frame, message, Color::Green);
    }
}

fn render_header(frame: &mut Frame, app: &App, area: Rect) {
    let current = app
        .current_session
        .as_ref()
        .map(|s| format!(" attached: {} ", s))
        .unwrap_or_default();

    let title_prefix = "─ tsm ─";
    let remaining_width = (area.width as usize).saturating_sub(title_prefix.len());
    let title = format!(
        "{}{:─>width$}",
        title_prefix,
        current,
        width = remaining_width
    );

    let header = Paragraph::new(title)
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD));

    frame.render_widget(header, area);
}

fn render_session_list(frame: &mut Frame, app: &mut App, area: Rect) {
    let selected_index = app.compute_flat_list_index();
    let total_items = app.compute_total_list_items();
    let visible_height = area.height as usize;

    let mut scroll_state = std::mem::take(&mut app.scroll_state);

    let filtered = app.filtered_sessions();

    if filtered.is_empty() {
        let empty_msg = if app.filter.is_empty() {
            "No tmux sessions found. Press 'n' to create one."
        } else {
            "No sessions match the filter."
        };
        let paragraph = Paragraph::new(empty_msg)
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center);
        frame.render_widget(paragraph, area);
        app.scroll_state = scroll_state;
        return;
    }

    // Calculate column widths
    let max_name_len = filtered
        .iter()
        .map(|s| s.name.width())
        .max()
        .unwrap_or(10)
        .max(10);

    let groups = app.grouped_sessions();

    let mut items: Vec<ListItem> = Vec::new();

    for (group_path, sessions) in &groups {
        // Render group header if there are multiple groups
        {
            let header_line = Line::from(vec![
                Span::styled(" ─ ", Style::default().fg(Color::DarkGray)),
                Span::styled(group_path, Style::default().fg(Color::Cyan)),
                Span::styled(
                    format!(" ({})", sessions.len()),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled(" ─", Style::default().fg(Color::DarkGray)),
            ]);
            items.push(ListItem::new(header_line));
        }

        for &(flat_idx, session) in sessions {
            let is_selected = flat_idx == app.selected;
            let is_current = app
                .current_session
                .as_ref()
                .is_some_and(|c| c == &session.name);

            let is_expanded = is_selected && matches!(app.mode, Mode::ActionMenu);
            let marker = if is_selected {
                if is_expanded { "▾" } else { "▸" }
            } else {
                " "
            };
            let status = &session.claude_code_status;

            let status_color = match (status, is_selected) {
                (ClaudeCodeStatus::Working, _) => Color::Green,
                (ClaudeCodeStatus::WaitingInput, _) => Color::Yellow,
                (ClaudeCodeStatus::Idle, true) => Color::White,
                (ClaudeCodeStatus::Idle, false) => Color::DarkGray,
                (ClaudeCodeStatus::Unknown, true) => Color::Gray,
                (ClaudeCodeStatus::Unknown, false) => Color::DarkGray,
            };

            let name_style = if is_current {
                Style::default().add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            // Order: name / status / pane_title / path
            let indent = "   ";
            let mut line_spans = vec![
                Span::raw(format!("{} {} ", indent, marker)),
                Span::styled(
                    format!("{:<width$}", session.name, width = max_name_len),
                    name_style,
                ),
                Span::raw("  "),
                Span::styled(status.symbol(), Style::default().fg(status_color)),
                Span::raw(" "),
                Span::styled(
                    format!("{:<8}", status.label()),
                    Style::default().fg(status_color),
                ),
            ];

            // Pane title
            if !session.pane_title.is_empty() {
                let title_color = if is_selected {
                    Color::Cyan
                } else {
                    Color::DarkGray
                };
                line_spans.push(Span::raw("  "));
                line_spans.push(Span::styled(
                    &session.pane_title,
                    Style::default().fg(title_color),
                ));
            }

            // Path is shown in group header, not repeated here

            let line = Line::from(line_spans);

            let style = if is_selected {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            };

            items.push(ListItem::new(line).style(style));

            if is_expanded {
                render_expanded_session_content(app, session, &mut items);
            }
        }
    }

    {
        let list = List::new(items);
        let list_state = scroll_state.update(selected_index, total_items, visible_height);
        StatefulWidget::render(list, area, frame.buffer_mut(), list_state);
    }

    app.scroll_state = scroll_state;
}

/// Render the expanded content for a session in action menu mode
fn render_expanded_session_content<'a>(
    app: &'a App,
    session: &'a crate::session::Session,
    items: &mut Vec<ListItem<'a>>,
) {
    let label_style = Style::default().fg(Color::DarkGray);
    let value_style = Style::default().fg(Color::White);

    // Session metadata row
    let attached_str = if session.attached { "yes" } else { "no" };
    let pane_count = session.panes.len();

    let meta_line = Line::from(vec![
        Span::raw("     "),
        Span::styled("windows: ", label_style),
        Span::styled(format!("{}", session.window_count), value_style),
        Span::raw("  "),
        Span::styled("panes: ", label_style),
        Span::styled(format!("{}", pane_count), value_style),
        Span::raw("  "),
        Span::styled("uptime: ", label_style),
        Span::styled(session.duration(), value_style),
        Span::raw("  "),
        Span::styled("attached: ", label_style),
        Span::styled(attached_str, value_style),
    ]);
    items.push(ListItem::new(meta_line));

    // Separator
    let sep_line = Line::from(Span::styled(
        "     ────────────────────────",
        Style::default().fg(Color::DarkGray),
    ));
    items.push(ListItem::new(sep_line));

    // Action items
    for (action_idx, action) in app.available_actions.iter().enumerate() {
        let is_action_selected = action_idx == app.selected_action;
        let action_marker = if is_action_selected { "▸" } else { " " };
        let action_style = if is_action_selected {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::White)
        };

        let action_line = Line::from(vec![
            Span::raw("     "),
            Span::styled(format!("{} {}", action_marker, action.label()), action_style),
        ]);
        items.push(ListItem::new(action_line));
    }

    // End separator
    let end_sep = Line::from(Span::styled("", Style::default().fg(Color::White)));
    items.push(ListItem::new(end_sep));
}

fn render_preview(frame: &mut Frame, app: &App, area: Rect) {
    frame.render_widget(Clear, area);

    // Draw separator lines at top and bottom
    let separator = "─".repeat(area.width as usize);

    let top_sep_area = Rect {
        x: area.x,
        y: area.y,
        width: area.width,
        height: 1,
    };
    let top_sep = Paragraph::new(separator.clone()).style(Style::default().fg(Color::DarkGray));
    frame.render_widget(top_sep, top_sep_area);

    let bottom_sep_area = Rect {
        x: area.x,
        y: area.y + area.height.saturating_sub(1),
        width: area.width,
        height: 1,
    };
    let bottom_sep = Paragraph::new(separator).style(Style::default().fg(Color::White));
    frame.render_widget(bottom_sep, bottom_sep_area);

    // Content area (between separators)
    let content_area = Rect {
        x: area.x,
        y: area.y + 1,
        width: area.width,
        height: area.height.saturating_sub(2),
    };

    let content = match &app.preview_content {
        Some(text) if !text.is_empty() => text,
        _ => {
            let msg = Paragraph::new("  No preview available")
                .style(Style::default().fg(Color::DarkGray));
            frame.render_widget(msg, content_area);
            return;
        }
    };

    // Parse ANSI escape sequences into styled ratatui Text
    let styled_text = match content.into_text() {
        Ok(text) => text,
        Err(_) => Text::raw(content),
    };

    // Take only the last N lines that fit in the content area
    let available_lines = content_area.height as usize;
    let total_lines = styled_text.lines.len();
    let start = total_lines.saturating_sub(available_lines);
    let visible_lines: Vec<Line> = styled_text.lines.into_iter().skip(start).collect();

    let preview = Paragraph::new(visible_lines);
    frame.render_widget(preview, content_area);
}

fn render_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    let (working, waiting, _idle) = app.status_counts();
    let total = app.sessions.len();

    let mut parts = vec![format!("{} sessions", total)];

    if working > 0 {
        parts.push(format!("{} working", working));
    }
    if waiting > 0 {
        parts.push(format!("{} awaiting input", waiting));
    }

    let status = parts.join(" │ ");

    let filter_info = if !app.filter.is_empty() {
        format!(" │ filter: \"{}\"", app.filter)
    } else {
        String::new()
    };

    let text = format!("  {}{}", status, filter_info);

    let bar = Paragraph::new(text).style(Style::default().fg(Color::DarkGray));

    frame.render_widget(bar, area);
}

fn render_footer(frame: &mut Frame, app: &App, area: Rect) {
    let hints = match app.mode {
        Mode::Normal => {
            "  ? help  jk navigate  l actions  ⏎ switch  ␣ peek  n new  K kill  p preview  / filter  q quit"
        }
        Mode::ActionMenu => "  jk navigate  ⏎/l select  h/esc back  q quit",
        Mode::Filter { .. } => "  ⏎ apply  esc cancel",
        Mode::ConfirmAction => "  y/⏎ confirm  n/esc cancel",
        Mode::NewSession { .. } => "  ⏎ create  tab switch  ↑↓ select  → accept  esc cancel",
        Mode::Rename { .. } => "  ⏎ confirm  esc cancel",
        Mode::Help => "  q close",
    };

    let footer = Paragraph::new(hints).style(Style::default().fg(Color::DarkGray));

    frame.render_widget(footer, area);
}

fn render_filter_bar(frame: &mut Frame, input: &str, area: Rect) {
    frame.render_widget(Clear, area);
    let text = format!("  / {}", input);
    let bar = Paragraph::new(text).style(Style::default().fg(Color::Yellow));
    frame.render_widget(bar, area);
}
