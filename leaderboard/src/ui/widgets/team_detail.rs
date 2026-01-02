use crate::state::achievements::AchievementType;
use crate::state::team::{ConsumerGroupStatus, TeamState};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

pub struct TeamDetailWidget<'a> {
    team: &'a TeamState,
    consumer_status: Option<&'a ConsumerGroupStatus>,
}

impl<'a> TeamDetailWidget<'a> {
    pub fn new(team: &'a TeamState, consumer_status: Option<&'a ConsumerGroupStatus>) -> Self {
        Self {
            team,
            consumer_status,
        }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        // Create centered popup (80% width, 85% height)
        let popup_area = centered_rect(80, 85, area);

        // Clear background
        frame.render_widget(Clear, popup_area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // Header
                Constraint::Length(7),  // Step achievements
                Constraint::Length(10), // Bonus achievements
                Constraint::Length(9),  // Stats
                Constraint::Min(3),     // Footer
            ])
            .split(popup_area);

        self.render_header(frame, chunks[0]);
        self.render_step_achievements(frame, chunks[1]);
        self.render_bonus_achievements(frame, chunks[2]);
        self.render_stats(frame, chunks[3]);
        self.render_footer(frame, chunks[4]);
    }

    fn render_header(&self, frame: &mut Frame, area: Rect) {
        let color = self.team_color();
        let header = Paragraph::new(format!("  {}", self.team.team_name))
            .style(
                Style::default()
                    .fg(color)
                    .add_modifier(Modifier::BOLD)
                    .bg(Color::Black),
            )
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Team Details ")
                    .style(Style::default().bg(Color::Black)),
            );
        frame.render_widget(header, area);
    }

    fn render_step_achievements(&self, frame: &mut Frame, area: Rect) {
        let mut lines = vec![Line::from(Span::styled(
            " Step Progress:",
            Style::default().add_modifier(Modifier::BOLD),
        ))];

        for step in AchievementType::all_steps() {
            let (status, style) = if self.team.has_achievement(step) {
                ("DONE", Style::default().fg(Color::Green))
            } else {
                ("pending", Style::default().fg(Color::Gray))
            };
            lines.push(Line::from(vec![
                Span::raw("   "),
                Span::styled(step.emoji(), style),
                Span::raw(" "),
                Span::styled(step.name(), style),
                Span::raw(" - "),
                Span::styled(status, style),
            ]));
        }

        let paragraph = Paragraph::new(lines).block(
            Block::default()
                .borders(Borders::LEFT | Borders::RIGHT)
                .style(Style::default().bg(Color::Black)),
        );
        frame.render_widget(paragraph, area);
    }

    fn render_bonus_achievements(&self, frame: &mut Frame, area: Rect) {
        let mut lines = vec![Line::from(Span::styled(
            " Bonus Achievements:",
            Style::default().add_modifier(Modifier::BOLD),
        ))];

        for bonus in AchievementType::all_bonus() {
            let (emoji, status, style) = if self.team.has_achievement(bonus) {
                (bonus.emoji(), "EARNED", Style::default().fg(Color::Magenta))
            } else {
                ("  ", "locked", Style::default().fg(Color::Gray))
            };
            lines.push(Line::from(vec![
                Span::raw("   "),
                Span::styled(emoji, style),
                Span::raw(" "),
                Span::styled(bonus.name(), style),
                Span::raw(" - "),
                Span::styled(status, style),
            ]));
        }

        let paragraph = Paragraph::new(lines).block(
            Block::default()
                .borders(Borders::LEFT | Borders::RIGHT)
                .style(Style::default().bg(Color::Black)),
        );
        frame.render_widget(paragraph, area);
    }

    fn render_stats(&self, frame: &mut Frame, area: Rect) {
        let consumers = self.consumer_status.map(|s| s.members).unwrap_or(0);
        let lag = self.consumer_status.map(|s| s.lag).unwrap_or(0);

        let stats = vec![
            Line::from(Span::styled(
                " Statistics:",
                Style::default().add_modifier(Modifier::BOLD),
            )),
            Line::from(format!("   Valid Actions:     {}", self.team.action_count)),
            Line::from(format!(
                "   Watchlist Entries: {}",
                self.team.watchlist_count
            )),
            Line::from(format!("   Consumer Members:  {}", consumers)),
            Line::from(format!("   Current Lag:       {}", lag)),
            Line::from(format!("   Max Lag Seen:      {}", self.team.max_lag_seen)),
            Line::from(format!(
                "   Parse Errors:      {}",
                self.team.get_error_count(AchievementType::ParseError)
            )),
            Line::from(format!(
                "   Missing Fields:    {}",
                self.team.get_error_count(AchievementType::MissingFields)
            )),
        ];

        let paragraph = Paragraph::new(stats).block(
            Block::default()
                .borders(Borders::LEFT | Borders::RIGHT)
                .style(Style::default().bg(Color::Black)),
        );
        frame.render_widget(paragraph, area);
    }

    fn render_footer(&self, frame: &mut Frame, area: Rect) {
        let footer = Paragraph::new(" Press Esc or Backspace to return")
            .style(Style::default().fg(Color::Gray).bg(Color::Black))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .style(Style::default().bg(Color::Black)),
            );
        frame.render_widget(footer, area);
    }

    fn team_color(&self) -> Color {
        match self.team.step_count() {
            4 => Color::Green,
            3 => Color::Yellow,
            2 => Color::Cyan,
            1 => Color::LightBlue,
            _ => Color::Gray,
        }
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_width = r.width * percent_x / 100;
    let popup_height = r.height * percent_y / 100;
    let popup_x = (r.width - popup_width) / 2;
    let popup_y = (r.height - popup_height) / 2;
    Rect::new(r.x + popup_x, r.y + popup_y, popup_width, popup_height)
}
