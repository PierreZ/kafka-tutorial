use crate::state::team::{ConsumerGroupStatus, GroupState};
use ratatui::{
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem},
    Frame,
};

pub struct ConsumerGroupsWidget<'a> {
    groups: &'a [ConsumerGroupStatus],
}

impl<'a> ConsumerGroupsWidget<'a> {
    pub fn new(groups: &'a [ConsumerGroupStatus]) -> Self {
        Self { groups }
    }

    pub fn render(&self, frame: &mut Frame, area: ratatui::layout::Rect) {
        let items: Vec<ListItem> = self
            .groups
            .iter()
            .map(|group| {
                let (status_icon, color) = match group.state {
                    GroupState::Active => ("●", Color::Green),
                    GroupState::Rebalancing => ("◐", Color::Yellow),
                    GroupState::Empty => ("○", Color::Gray),
                    GroupState::Unknown => ("?", Color::DarkGray),
                };

                let members_str = if group.members > 0 {
                    format!("({})", group.members)
                } else {
                    String::new()
                };

                let line = Line::from(vec![
                    Span::styled(format!("{} ", status_icon), Style::default().fg(color)),
                    Span::raw(&group.team_name),
                    Span::raw(": "),
                    Span::styled(format!("{}", group.state), Style::default().fg(color)),
                    Span::styled(members_str, Style::default().fg(Color::Cyan)),
                ]);

                ListItem::new(line)
            })
            .collect();

        let list = List::new(items).block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Consumer Groups "),
        );

        frame.render_widget(list, area);
    }
}
