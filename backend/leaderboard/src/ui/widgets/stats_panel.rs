//! Stats panel widget - shows live stats from the compacted team_stats topic

use crate::state::TeamState;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use std::collections::HashMap;

/// Data needed for rendering the stats panel
pub struct StatsPanelData<'a> {
    pub teams: &'a HashMap<String, TeamState>,
    pub sorted_teams: &'a [String],
}

/// Render the stats panel showing live stats from compacted topic
pub fn render(frame: &mut Frame, area: Rect, data: &StatsPanelData) {
    let mut lines: Vec<Line> = Vec::new();

    // Header
    lines.push(Line::from(vec![
        Span::styled(
            " Team     ",
            Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            "Proc  ",
            Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            "Flag  ",
            Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            "Rate ",
            Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        ),
    ]));

    // Team rows
    for team_name in data.sorted_teams {
        if let Some(team) = data.teams.get(team_name) {
            let (proc_str, flag_str, rate_str) = if let Some(ref stats) = team.latest_stats {
                let rate = if stats.processed > 0 {
                    (stats.flagged as f64 / stats.processed as f64 * 100.0) as u32
                } else {
                    0
                };
                (
                    format!("{:>5}", stats.processed),
                    format!("{:>5}", stats.flagged),
                    format!("{:>3}%", rate),
                )
            } else {
                (
                    "    -".to_string(),
                    "    -".to_string(),
                    "   -".to_string(),
                )
            };

            // Color based on whether they have stats
            let has_stats = team.latest_stats.is_some();
            let style = if has_stats {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::DarkGray)
            };

            // Short team name (remove "team-" prefix for compactness)
            let short_name = team_name.strip_prefix("team-").unwrap_or(team_name);

            lines.push(Line::from(vec![
                Span::styled(format!(" {:<8}", short_name), style),
                Span::styled(format!("{} ", proc_str), style),
                Span::styled(format!("{} ", flag_str), style),
                Span::styled(format!("{} ", rate_str), style),
            ]));
        }
    }

    let content = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray))
            .title(Line::from(vec![Span::styled(
                " Live Stats ",
                Style::default().fg(Color::White),
            )])),
    );

    frame.render_widget(content, area);
}
