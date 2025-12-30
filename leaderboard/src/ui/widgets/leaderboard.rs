use crate::state::achievements::AchievementType;
use crate::state::team::{ConsumerGroupStatus, TeamState};
use ratatui::{
    layout::Constraint,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Row, Table},
    Frame,
};
use std::collections::HashMap;

pub struct LeaderboardWidget<'a> {
    teams: &'a [TeamState],
    consumer_groups: &'a HashMap<String, u32>, // team_name -> member count
}

impl<'a> LeaderboardWidget<'a> {
    pub fn new(teams: &'a [TeamState], consumer_groups: &'a HashMap<String, u32>) -> Self {
        Self {
            teams,
            consumer_groups,
        }
    }

    /// Build consumer groups map from status list
    pub fn build_consumer_map(statuses: &[ConsumerGroupStatus]) -> HashMap<String, u32> {
        statuses
            .iter()
            .map(|s| (s.team_name.clone(), s.members))
            .collect()
    }

    pub fn render(&self, frame: &mut Frame, area: ratatui::layout::Rect) {
        let header = Row::new(vec![
            Cell::from("Team"),
            Cell::from("Progress"),
            Cell::from("Errors"),
            Cell::from("👥"),
            Cell::from("📤"),
        ])
        .style(Style::default().add_modifier(Modifier::BOLD))
        .height(1);

        // Sort teams by step count (descending), then by action count, then by name
        let mut sorted_teams: Vec<_> = self.teams.iter().collect();
        sorted_teams.sort_by(|a, b| {
            b.step_count()
                .cmp(&a.step_count())
                .then_with(|| b.action_count.cmp(&a.action_count))
                .then_with(|| a.team_name.cmp(&b.team_name))
        });

        let rows: Vec<Row> = sorted_teams
            .iter()
            .map(|team| {
                // Progress emojis
                let progress = self.format_progress(team);

                // Error emojis with counts
                let errors = self.format_errors(team);

                // Consumer count
                let consumers = self.consumer_groups.get(&team.team_name).unwrap_or(&0);

                // Color based on progress
                let color = match team.step_count() {
                    4 => Color::Green,    // All done
                    3 => Color::Yellow,   // Almost there
                    2 => Color::Cyan,     // Making progress
                    1 => Color::Blue,     // Started
                    _ => Color::DarkGray, // Not started
                };

                Row::new(vec![
                    Cell::from(team.team_name.clone()),
                    Cell::from(progress),
                    Cell::from(errors),
                    Cell::from(format!("{}", consumers)),
                    Cell::from(format!("{}", team.action_count)),
                ])
                .style(Style::default().fg(color))
            })
            .collect();

        let widths = [
            Constraint::Length(10), // Team
            Constraint::Length(16), // Progress (4 emojis + spaces)
            Constraint::Length(12), // Errors
            Constraint::Length(4),  // 👥
            Constraint::Length(8),  // 📤
        ];

        let table = Table::new(rows, widths).header(header).block(
            Block::default()
                .borders(Borders::ALL)
                .title(" KAFKA TUTORIAL - 1️⃣=Connected 3️⃣=Load 4️⃣=Scaled 5️⃣=Watchlist "),
        );

        frame.render_widget(table, area);
    }

    fn format_progress(&self, team: &TeamState) -> String {
        let mut progress = String::new();
        for step in AchievementType::all_steps() {
            if team.has_achievement(step) {
                progress.push_str(step.emoji());
            } else {
                progress.push_str("⬜");
            }
            progress.push(' ');
        }
        progress.trim_end().to_string()
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
