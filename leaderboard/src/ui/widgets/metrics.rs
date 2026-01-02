use crate::state::team::{ConsumerGroupStatus, GroupState, TeamState};
use ratatui::{
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Row, Table, TableState},
    Frame,
};
use std::collections::HashMap;
use std::time::Instant;

pub struct MetricsWidget<'a> {
    teams: &'a mut [TeamState],
    consumer_groups: &'a HashMap<String, ConsumerGroupStatus>,
    last_update: Option<Instant>,
    selected_index: Option<usize>,
    total_teams: usize,
}

impl<'a> MetricsWidget<'a> {
    pub fn new(
        teams: &'a mut [TeamState],
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

    /// Build consumer groups map from status list
    pub fn build_consumer_map(
        statuses: &[ConsumerGroupStatus],
    ) -> HashMap<String, ConsumerGroupStatus> {
        statuses
            .iter()
            .map(|s| (s.team_name.clone(), s.clone()))
            .collect()
    }

    /// Render with stateful selection support
    pub fn render_stateful(&mut self, frame: &mut Frame, area: Rect, table_state: &mut TableState) {
        let header = Row::new(vec![
            Cell::from("Team"),
            Cell::from("State"),
            Cell::from("👥"),
            Cell::from("📥/m"),
            Cell::from("📤/m"),
            Cell::from("Lag"),
            Cell::from("MaxLag"),
            Cell::from("Last"),
            Cell::from("Session"),
        ])
        .style(Style::default().add_modifier(Modifier::BOLD))
        .height(1);

        let rows: Vec<Row> = self
            .teams
            .iter_mut()
            .map(|team| {
                let group = self.consumer_groups.get(&team.team_name);

                // Consumer group state
                let (state_str, state_color) = match group.map(|g| &g.state) {
                    Some(GroupState::Active) => ("Active", Color::Green),
                    Some(GroupState::Rebalancing) => ("Rebalancing", Color::Yellow),
                    Some(GroupState::Empty) => ("Empty", Color::Gray),
                    Some(GroupState::Unknown) | None => ("Unknown", Color::Gray),
                };

                // Member count
                let members = group.map(|g| g.members).unwrap_or(0);

                // Rates (msgs per minute)
                let consumption_rate = team.consumption_rate();
                let production_rate = team.production_rate();

                // Lag coloring
                let lag = team.current_lag;
                let lag_color = if lag <= 10 {
                    Color::Green
                } else if lag <= 50 {
                    Color::Yellow
                } else {
                    Color::Red
                };

                // Last activity
                let last_activity = team.last_activity_display();

                // Session duration
                let session = team.session_duration_display();

                // Team name color based on activity
                let team_color = if production_rate > 0 {
                    Color::White
                } else if consumption_rate > 0 {
                    Color::Cyan
                } else {
                    Color::White
                };

                Row::new(vec![
                    Cell::from(team.team_name.clone()).style(Style::default().fg(team_color)),
                    Cell::from(state_str).style(Style::default().fg(state_color)),
                    Cell::from(format!("{}", members)),
                    Cell::from(format!("{}", consumption_rate)),
                    Cell::from(format!("{}", production_rate)),
                    Cell::from(format!("{}", lag)).style(Style::default().fg(lag_color)),
                    Cell::from(format!("{}", team.max_lag_seen)),
                    Cell::from(last_activity),
                    Cell::from(session),
                ])
            })
            .collect();

        let widths = [
            Constraint::Length(10), // Team
            Constraint::Length(12), // State
            Constraint::Length(4),  // 👥
            Constraint::Length(6),  // 📥/m
            Constraint::Length(6),  // 📤/m
            Constraint::Length(6),  // Lag
            Constraint::Length(8),  // MaxLag
            Constraint::Length(6),  // Last
            Constraint::Length(10), // Session
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

        format!(" METRICS - Rates & Lag | {} | {} ", position, freshness)
    }
}
