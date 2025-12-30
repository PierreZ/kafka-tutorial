use crate::state::team::TeamState;
use ratatui::{
    layout::Constraint,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Row, Table},
    Frame,
};

pub struct LeaderboardWidget<'a> {
    teams: &'a [TeamState],
}

impl<'a> LeaderboardWidget<'a> {
    pub fn new(teams: &'a [TeamState]) -> Self {
        Self { teams }
    }

    pub fn render(&self, frame: &mut Frame, area: ratatui::layout::Rect) {
        let header = Row::new(vec![
            Cell::from("#"),
            Cell::from("Team"),
            Cell::from("Score"),
            Cell::from("C/I"),
            Cell::from("Streak"),
            Cell::from("Best"),
        ])
        .style(Style::default().add_modifier(Modifier::BOLD))
        .height(1);

        // Sort teams by score (descending)
        let mut sorted_teams: Vec<_> = self.teams.iter().collect();
        sorted_teams.sort_by(|a, b| b.score.cmp(&a.score));

        let rows: Vec<Row> = sorted_teams
            .iter()
            .enumerate()
            .map(|(idx, team)| {
                let rank = idx + 1;
                let color = match rank {
                    1 => Color::Yellow,
                    2 => Color::White,
                    3 => Color::Rgb(205, 127, 50), // Bronze
                    _ => Color::Gray,
                };

                Row::new(vec![
                    Cell::from(format!("{}", rank)),
                    Cell::from(team.team_name.clone()),
                    Cell::from(format!("{}", team.score)),
                    Cell::from(format!("{}/{}", team.correct_count, team.incorrect_count)),
                    Cell::from(format!("{}", team.current_streak)),
                    Cell::from(format!("{}", team.best_streak)),
                ])
                .style(Style::default().fg(color))
            })
            .collect();

        let widths = [
            Constraint::Length(3),
            Constraint::Length(10),
            Constraint::Length(8),
            Constraint::Length(10),
            Constraint::Length(7),
            Constraint::Length(6),
        ];

        let table = Table::new(rows, widths)
            .header(header)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Team Rankings "),
            )
            .row_highlight_style(Style::default().add_modifier(Modifier::REVERSED));

        frame.render_widget(table, area);
    }
}
