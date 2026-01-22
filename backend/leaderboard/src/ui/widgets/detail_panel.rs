//! Detail panel widget - side panel showing selected team details

use crate::state::{AchievementType, TeamState, STEP_INCOMPLETE};
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

/// Data needed for rendering the detail panel
pub struct DetailPanelData<'a> {
    pub team_state: &'a TeamState,
    pub consumer_count: u32,
}

/// Render the detail panel
pub fn render(frame: &mut Frame, area: Rect, data: &DetailPanelData) {
    let mut lines: Vec<Line> = Vec::new();

    // Team header
    lines.push(Line::from(vec![Span::styled(
        format!(" {} ", data.team_state.team_name.to_uppercase()),
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    )]));
    lines.push(Line::from(""));

    // Step Progress section
    lines.push(Line::from(vec![Span::styled(
        "Progress",
        Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD),
    )]));

    for step in AchievementType::all_steps() {
        let has_it = data.team_state.has_achievement(step);
        let emoji = if has_it {
            step.emoji()
        } else {
            STEP_INCOMPLETE
        };
        let status = if has_it { "OK" } else { "  " };

        let time_info = if has_it {
            data.team_state
                .achievement_timestamps
                .get(&step)
                .map(|t| {
                    let duration = chrono::Utc::now().signed_duration_since(*t);
                    let mins = duration.num_minutes();
                    if mins < 60 {
                        format!("{}m", mins)
                    } else {
                        format!("{}h", mins / 60)
                    }
                })
                .unwrap_or_default()
        } else {
            "".to_string()
        };

        let style = if has_it {
            Style::default().fg(Color::Green)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        lines.push(Line::from(vec![
            Span::raw(" "),
            Span::styled(format!("{} {:<12}", emoji, step.name()), style),
            Span::styled(format!("{:>2}", status), style),
            Span::raw(" "),
            Span::styled(time_info, Style::default().fg(Color::DarkGray)),
        ]));
    }

    lines.push(Line::from(""));

    // Stats section
    lines.push(Line::from(vec![Span::styled(
        "Stats",
        Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD),
    )]));

    let stats = [
        ("Actions", format!("{}", data.team_state.action_count)),
        ("Stats", format!("{}", data.team_state.stats_count)),
        ("Consumers", format!("{}", data.consumer_count)),
        ("Lag", format!("{}", data.team_state.current_lag)),
        ("Session", data.team_state.session_duration_display()),
    ];

    for (label, value) in stats {
        lines.push(Line::from(vec![
            Span::raw(" "),
            Span::styled(
                format!("{:<10}", label),
                Style::default().fg(Color::DarkGray),
            ),
            Span::styled(value, Style::default().fg(Color::White)),
        ]));
    }

    let content = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray))
                .title(Line::from(vec![Span::styled(
                    " Details ",
                    Style::default().fg(Color::White),
                )])),
        )
        .wrap(Wrap { trim: false });

    frame.render_widget(content, area);
}

/// Render an empty detail panel with instructions
pub fn render_empty(frame: &mut Frame, area: Rect) {
    let lines = vec![
        Line::from(""),
        Line::from(""),
        Line::from(vec![Span::styled(
            "  Select a team",
            Style::default().fg(Color::DarkGray),
        )]),
        Line::from(vec![Span::styled(
            "  to view details",
            Style::default().fg(Color::DarkGray),
        )]),
    ];

    let content = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray))
            .title(Line::from(vec![Span::styled(
                " Details ",
                Style::default().fg(Color::White),
            )])),
    );

    frame.render_widget(content, area);
}
