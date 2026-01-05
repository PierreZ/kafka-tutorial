use crate::state::achievements::AchievementType;
use crate::state::team::{ConsumerGroupStatus, GroupState, TeamState};
use ratatui::{
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Row, Table, TableState},
    Frame,
};
use std::collections::HashMap;
use std::time::Instant;

pub struct LeaderboardWidget<'a> {
    teams: &'a [TeamState],
    consumer_groups: &'a HashMap<String, ConsumerGroupStatus>,
    last_update: Option<Instant>,
    selected_index: Option<usize>,
    total_teams: usize,
}

impl<'a> LeaderboardWidget<'a> {
    pub fn new(
        teams: &'a [TeamState],
        consumer_groups: &'a HashMap<String, ConsumerGroupStatus>,
        last_update: Option<Instant>,
        selected_index: Option<usize>,
        total_teams: usize,
    ) -> Self {
        Self {
            teams,
            consumer_groups,
            last_update,
            selected_index,
            total_teams,
        }
    }

    /// Build consumer groups map from status list (full status version)
    pub fn build_consumer_map(
        statuses: &[ConsumerGroupStatus],
    ) -> HashMap<String, ConsumerGroupStatus> {
        statuses
            .iter()
            .map(|s| (s.team_name.clone(), s.clone()))
            .collect()
    }

    /// Render with stateful selection support
    pub fn render_stateful(&self, frame: &mut Frame, area: Rect, table_state: &mut TableState) {
        let header = Row::new(vec![
            Cell::from("Team"),
            Cell::from("S1"),
            Cell::from("S2"),
            Cell::from("S3"),
            Cell::from("S4"),
            Cell::from("Bonus"),
            Cell::from("Consumers"),
            Cell::from("Lag"),
            Cell::from("Last actions"),
            Cell::from("Last watchlist"),
        ])
        .style(Style::default().add_modifier(Modifier::BOLD))
        .height(1);

        // Teams should already be sorted by caller
        let rows: Vec<Row> = self
            .teams
            .iter()
            .map(|team| {
                let steps = AchievementType::all_steps();

                // Step checkmarks
                let s1 = self.format_step_checkmark(team, steps[0]);
                let s2 = self.format_step_checkmark(team, steps[1]);
                let s3 = self.format_step_checkmark(team, steps[2]);
                let s4 = self.format_step_checkmark(team, steps[3]);

                // Bonus achievements (compact)
                let bonus = self.format_bonus_compact(team);

                // Consumer count and lag from consumer group
                let (consumers, lag, lag_color) = self.format_consumer_info(&team.team_name);

                // Last activity times
                let last_actions = team.last_activity_display();
                let last_watchlist = team.last_watchlist_display();

                // Row color based on progress
                let row_color = match team.step_count() {
                    4 => Color::Green,     // All done
                    3 => Color::Yellow,    // Almost there
                    2 => Color::Cyan,      // Making progress
                    1 => Color::LightBlue, // Started
                    _ => Color::Gray,      // Not started
                };

                Row::new(vec![
                    Cell::from(team.team_name.clone()),
                    Cell::from(s1.0).style(Style::default().fg(s1.1)),
                    Cell::from(s2.0).style(Style::default().fg(s2.1)),
                    Cell::from(s3.0).style(Style::default().fg(s3.1)),
                    Cell::from(s4.0).style(Style::default().fg(s4.1)),
                    Cell::from(bonus),
                    Cell::from(consumers),
                    Cell::from(lag).style(Style::default().fg(lag_color)),
                    Cell::from(last_actions),
                    Cell::from(last_watchlist),
                ])
                .style(Style::default().fg(row_color))
            })
            .collect();

        let widths = [
            Constraint::Length(10), // Team
            Constraint::Length(2),  // S1
            Constraint::Length(2),  // S2
            Constraint::Length(2),  // S3
            Constraint::Length(2),  // S4
            Constraint::Length(12), // Bonus
            Constraint::Length(9),  // Consumers
            Constraint::Length(4),  // Lag
            Constraint::Length(12), // Last actions
            Constraint::Length(14), // Last watchlist
        ];

        let title = self.build_title();

        let table = Table::new(rows, widths)
            .header(header)
            .block(Block::default().borders(Borders::ALL).title(title))
            .row_highlight_style(
                Style::default()
                    .bg(Color::Gray)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(">> ");

        frame.render_stateful_widget(table, area, table_state);
    }

    fn build_title(&self) -> String {
        // Count active teams
        let active_count = self
            .consumer_groups
            .values()
            .filter(|s| matches!(s.state, GroupState::Active))
            .count();

        let freshness = match self.last_update {
            Some(instant) => {
                let elapsed = instant.elapsed().as_secs();
                if elapsed < 2 {
                    "LIVE".to_string()
                } else if elapsed < 60 {
                    format!("{}s ago", elapsed)
                } else {
                    format!("{}m ago", elapsed / 60)
                }
            }
            None => "Connecting...".to_string(),
        };

        let position = match self.selected_index {
            Some(idx) => format!("{}/{}", idx + 1, self.total_teams),
            None => format!("-/{}", self.total_teams),
        };

        format!(
            " KAFKA LEADERBOARD | {}/{} active | {} | {} ",
            active_count, self.total_teams, position, freshness
        )
    }

    /// Format step achievement as checkmark or dot
    fn format_step_checkmark(&self, team: &TeamState, step: AchievementType) -> (String, Color) {
        if team.has_achievement(step) {
            ("✓".to_string(), Color::Green)
        } else {
            ("·".to_string(), Color::Gray)
        }
    }

    /// Format bonus achievements compactly (only show earned emojis)
    fn format_bonus_compact(&self, team: &TeamState) -> String {
        let mut bonus = String::new();
        for b in AchievementType::all_bonus() {
            if team.has_achievement(b) {
                bonus.push_str(b.emoji());
            }
        }
        bonus
    }

    /// Get consumer count, lag, and lag color from consumer group status
    fn format_consumer_info(&self, team_name: &str) -> (String, String, Color) {
        match self.consumer_groups.get(team_name) {
            Some(status) => {
                let consumers = format!("{}", status.members);
                let lag = format!("{}", status.lag);
                let lag_color = if status.lag <= 10 {
                    Color::Green
                } else if status.lag <= 50 {
                    Color::Yellow
                } else {
                    Color::Red
                };
                (consumers, lag, lag_color)
            }
            None => ("0".to_string(), "-".to_string(), Color::Gray),
        }
    }
}
