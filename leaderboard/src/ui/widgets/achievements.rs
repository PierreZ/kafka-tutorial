use crate::state::achievements::AchievementType;
use chrono::{DateTime, Utc};
use ratatui::{
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem},
    Frame,
};
use std::collections::VecDeque;

/// Recent achievement notification
#[derive(Debug, Clone)]
pub struct AchievementNotification {
    pub team_name: String,
    pub achievement: AchievementType,
    pub timestamp: DateTime<Utc>,
}

pub struct AchievementsWidget<'a> {
    notifications: &'a VecDeque<AchievementNotification>,
    max_display: usize,
}

impl<'a> AchievementsWidget<'a> {
    pub fn new(notifications: &'a VecDeque<AchievementNotification>, max_display: usize) -> Self {
        Self {
            notifications,
            max_display,
        }
    }

    pub fn render(&self, frame: &mut Frame, area: ratatui::layout::Rect) {
        let items: Vec<ListItem> = self
            .notifications
            .iter()
            .take(self.max_display)
            .map(|notif| {
                let time = notif.timestamp.format("%H:%M:%S").to_string();
                let color = if notif.achievement.is_mistake() {
                    Color::Red
                } else {
                    Color::Green
                };

                let line = Line::from(vec![
                    Span::styled(format!("[{}] ", time), Style::default().fg(Color::DarkGray)),
                    Span::styled(&notif.team_name, Style::default().fg(Color::Cyan)),
                    Span::raw(": "),
                    Span::styled(
                        format!("{}", notif.achievement),
                        Style::default().fg(color),
                    ),
                ]);

                ListItem::new(line)
            })
            .collect();

        let list = List::new(items).block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Recent Achievements "),
        );

        frame.render_widget(list, area);
    }
}
