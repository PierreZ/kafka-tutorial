//! Leaderboard table widget - compact table with all teams

use crate::state::{AchievementType, TeamState, STEP_INCOMPLETE};
use ratatui::{
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Row, Table, TableState},
    Frame,
};
use std::collections::HashMap;

/// Data for rendering the leaderboard table
pub struct LeaderboardTableData<'a> {
    pub teams: &'a HashMap<String, TeamState>,
    pub consumer_counts: &'a HashMap<String, u32>,
    pub selected_row: usize,
    pub celebration_team: Option<&'a str>,
}

/// Get sorted team names (by progress, then action count, then name)
pub fn get_sorted_teams(teams: &HashMap<String, TeamState>) -> Vec<String> {
    let mut team_names: Vec<_> = teams.keys().cloned().collect();
    team_names.sort_by(|a, b| {
        let state_a = teams.get(a);
        let state_b = teams.get(b);

        match (state_a, state_b) {
            (Some(sa), Some(sb)) => {
                // Primary: step count descending
                let step_cmp = sb.step_count().cmp(&sa.step_count());
                if step_cmp != std::cmp::Ordering::Equal {
                    return step_cmp;
                }
                // Secondary: action count descending
                let action_cmp = sb.action_count.cmp(&sa.action_count);
                if action_cmp != std::cmp::Ordering::Equal {
                    return action_cmp;
                }
                // Tertiary: team name ascending
                a.cmp(b)
            }
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => a.cmp(b),
        }
    });
    team_names
}

/// Get progress emoji string for a team
fn get_progress_emojis(state: &TeamState) -> String {
    let mut progress = String::new();
    for step in AchievementType::all_steps() {
        if state.has_achievement(step) {
            progress.push_str(step.emoji());
        } else {
            progress.push_str(STEP_INCOMPLETE);
        }
    }
    progress
}

/// Get row color based on step progress
fn get_row_color(step_count: usize) -> Color {
    match step_count {
        4 => Color::Green,
        3 => Color::Yellow,
        2 => Color::Cyan,
        1 => Color::LightBlue,
        _ => Color::DarkGray,
    }
}

/// Format lag value compactly
fn format_lag(lag: i64, connected: bool) -> String {
    if !connected {
        return "-".to_string();
    }
    if lag >= 1000 {
        format!("{:.1}k", lag as f64 / 1000.0)
    } else {
        format!("{}", lag)
    }
}

/// Render the leaderboard table
pub fn render(
    frame: &mut Frame,
    area: Rect,
    data: &LeaderboardTableData,
    sorted_teams: &[String],
    table_state: &mut TableState,
) {
    // Build rows
    let rows: Vec<Row> = sorted_teams
        .iter()
        .enumerate()
        .map(|(idx, team_name)| {
            let rank = idx + 1;
            let state = data.teams.get(team_name);
            let consumer_count = data.consumer_counts.get(team_name).copied().unwrap_or(0);

            let (progress, lag, actions, stats, step_count) = match state {
                Some(s) => {
                    let progress = get_progress_emojis(s);
                    let connected = s.has_achievement(AchievementType::Connected);
                    let lag = format_lag(s.current_lag, connected);
                    let actions = if s.action_count > 0 {
                        format!("{}", s.action_count)
                    } else {
                        "-".to_string()
                    };
                    let stats = if s.stats_count > 0 {
                        format!("{}", s.stats_count)
                    } else {
                        "-".to_string()
                    };
                    (progress, lag, actions, stats, s.step_count())
                }
                None => {
                    let progress = format!(
                        "{}{}{}{}",
                        STEP_INCOMPLETE, STEP_INCOMPLETE, STEP_INCOMPLETE, STEP_INCOMPLETE
                    );
                    (
                        progress,
                        "-".to_string(),
                        "-".to_string(),
                        "-".to_string(),
                        0,
                    )
                }
            };

            // Row styling
            let is_selected = data.selected_row == idx;
            let is_celebrating = data.celebration_team == Some(team_name.as_str());
            let base_color = get_row_color(step_count);

            let style = if is_celebrating {
                Style::default()
                    .bg(Color::White)
                    .fg(Color::Black)
                    .add_modifier(Modifier::BOLD)
            } else if is_selected {
                Style::default()
                    .bg(Color::DarkGray)
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(base_color)
            };

            // Build cells - rank with selection marker
            let rank_display = if is_selected {
                format!(">{}", rank)
            } else {
                format!("{:>2}", rank)
            };

            Row::new(vec![
                Cell::from(rank_display),
                Cell::from(team_name.clone()),
                Cell::from(progress),
                Cell::from(format!("{:>2}", consumer_count)),
                Cell::from(format!("{:>5}", lag)),
                Cell::from(format!("{:>5}", actions)),
                Cell::from(format!("{:>5}", stats)),
            ])
            .style(style)
        })
        .collect();

    // Column widths - optimized for compact display
    let widths = [
        Constraint::Length(3),  // Rank
        Constraint::Length(8),  // Team
        Constraint::Length(9),  // Progress (4 emojis)
        Constraint::Length(4),  // Instances
        Constraint::Length(6),  // Lag
        Constraint::Length(6),  // Actions
        Constraint::Min(6),     // Stats (flexible)
    ];

    // Header
    let header = Row::new(vec![
        Cell::from(" #"),
        Cell::from("Team"),
        Cell::from("Progress"),
        Cell::from("Inst"),
        Cell::from("  Lag"),
        Cell::from("Action"),
        Cell::from(" Stats"),
    ])
    .style(
        Style::default()
            .add_modifier(Modifier::BOLD)
            .fg(Color::DarkGray),
    )
    .height(1);

    let table = Table::new(rows, widths)
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray))
                .title(Line::from(vec![Span::styled(
                    " Teams ",
                    Style::default().fg(Color::White),
                )])),
        )
        .row_highlight_style(Style::default());

    frame.render_stateful_widget(table, area, table_state);
}
