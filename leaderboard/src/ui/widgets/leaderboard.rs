use crate::state::achievements::AchievementType;
use crate::state::team::{ConsumerGroupStatus, TeamState};
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
    consumer_groups: &'a HashMap<String, u32>, // team_name -> member count
    last_update: Option<Instant>,
    selected_index: Option<usize>,
    total_teams: usize,
}

impl<'a> LeaderboardWidget<'a> {
    pub fn new(
        teams: &'a [TeamState],
        consumer_groups: &'a HashMap<String, u32>,
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

    /// Build consumer groups map from status list
    pub fn build_consumer_map(statuses: &[ConsumerGroupStatus]) -> HashMap<String, u32> {
        statuses
            .iter()
            .map(|s| (s.team_name.clone(), s.members))
            .collect()
    }

    /// Render with stateful selection support
    pub fn render_stateful(&self, frame: &mut Frame, area: Rect, table_state: &mut TableState) {
        let header = Row::new(vec![
            Cell::from("Team"),
            Cell::from("Achievements"),
            Cell::from("Errors"),
            Cell::from("👥"),
            Cell::from("📤"),
        ])
        .style(Style::default().add_modifier(Modifier::BOLD))
        .height(1);

        // Teams should already be sorted by caller
        let rows: Vec<Row> = self
            .teams
            .iter()
            .map(|team| {
                // All achievements (steps + bonus)
                let achievements = self.format_all_achievements(team);

                // Error emojis with counts
                let errors = self.format_errors(team);

                // Consumer count
                let consumers = self.consumer_groups.get(&team.team_name).unwrap_or(&0);

                // Color based on progress
                let color = match team.step_count() {
                    4 => Color::Green,     // All done
                    3 => Color::Yellow,    // Almost there
                    2 => Color::Cyan,      // Making progress
                    1 => Color::LightBlue, // Started
                    _ => Color::Gray,      // Not started
                };

                Row::new(vec![
                    Cell::from(team.team_name.clone()),
                    Cell::from(achievements),
                    Cell::from(errors),
                    Cell::from(format!("{}", consumers)),
                    Cell::from(format!("{}", team.action_count)),
                ])
                .style(Style::default().fg(color))
            })
            .collect();

        let widths = [
            Constraint::Length(10), // Team
            Constraint::Length(32), // Achievements (steps + bonus emojis)
            Constraint::Length(12), // Errors
            Constraint::Length(4),  // 👥
            Constraint::Length(8),  // 📤
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
            " KAFKA TUTORIAL - Steps: 🔗📤⚡👁 | Bonus: 🔬📈✨⚔🚀🏆 | {} | {} ",
            position, freshness
        )
    }

    fn format_all_achievements(&self, team: &TeamState) -> String {
        let mut achievements = String::new();

        // Step achievements: show emoji if achieved, dot if not
        for step in AchievementType::all_steps() {
            if team.has_achievement(step) {
                achievements.push_str(step.emoji());
            } else {
                achievements.push('·');
            }
        }

        // Add separator if team has any bonus achievements
        let has_bonus = AchievementType::all_bonus()
            .iter()
            .any(|a| team.has_achievement(*a));
        if has_bonus {
            achievements.push(' ');
        }

        // Bonus achievements (only show if earned)
        for bonus in AchievementType::all_bonus() {
            if team.has_achievement(bonus) {
                achievements.push_str(bonus.emoji());
            }
        }

        achievements
    }

    fn format_errors(&self, team: &TeamState) -> String {
        let mut errors = String::new();
        for error in AchievementType::all_errors() {
            let count = team.get_error_count(error);
            if count > 0 {
                errors.push_str(error.emoji());
                errors.push_str(&format!("x{} ", count));
            }
        }
        errors.trim_end().to_string()
    }
}
