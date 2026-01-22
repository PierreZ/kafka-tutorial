//! Help overlay widget - centered popup with keybindings and achievement info

use crate::state::AchievementType;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

/// Render the help overlay
pub fn render(frame: &mut Frame) {
    let area = frame.area();

    // Center the popup (60% width, 80% height)
    let popup_width = (area.width * 60) / 100;
    let popup_height = (area.height * 80) / 100;
    let popup_x = (area.width - popup_width) / 2;
    let popup_y = (area.height - popup_height) / 2;

    let popup_area = Rect::new(popup_x, popup_y, popup_width, popup_height);

    // Clear the background
    frame.render_widget(Clear, popup_area);

    let mut lines: Vec<Line> = Vec::new();

    // Title
    lines.push(Line::from(vec![Span::styled(
        " HELP ",
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    )]));
    lines.push(Line::from(""));

    // Navigation keys
    lines.push(Line::from(vec![Span::styled(
        "Navigation",
        Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD),
    )]));

    let nav_keys = [
        ("j / k", "Move down / up"),
        ("g / G", "Jump to first / last"),
        ("d", "Toggle detail panel"),
        ("f", "Toggle fullscreen"),
        ("Enter", "Toggle detail panel"),
        ("?", "Show/hide help"),
        ("q / Esc", "Quit"),
    ];

    for (key, desc) in nav_keys {
        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled(format!("{:>12}", key), Style::default().fg(Color::Yellow)),
            Span::raw("  "),
            Span::styled(desc, Style::default().fg(Color::Gray)),
        ]));
    }

    lines.push(Line::from(""));

    // Step achievements
    lines.push(Line::from(vec![Span::styled(
        "Step Achievements",
        Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD),
    )]));

    for step in AchievementType::all_steps() {
        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled(
                format!("{} {:<15}", step.emoji(), step.name()),
                Style::default().fg(Color::Green),
            ),
            Span::styled(step.description(), Style::default().fg(Color::DarkGray)),
        ]));
    }

    lines.push(Line::from(""));

    // Status icons legend
    lines.push(Line::from(vec![Span::styled(
        "Status Icons",
        Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD),
    )]));

    let status_icons = [
        ("OK", "Connected and active", Color::Green),
        ("Disconnected", "Consumer group empty", Color::Yellow),
        ("Pending", "Not yet connected", Color::DarkGray),
    ];

    for (icon, desc, color) in status_icons {
        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled(format!("{:<15}", icon), Style::default().fg(color)),
            Span::styled(desc, Style::default().fg(Color::DarkGray)),
        ]));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(vec![Span::styled(
        "  Press ? or Esc to close",
        Style::default().fg(Color::DarkGray),
    )]));

    let content = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan))
                .style(Style::default().bg(Color::Black)),
        )
        .wrap(Wrap { trim: false });

    frame.render_widget(content, popup_area);
}
