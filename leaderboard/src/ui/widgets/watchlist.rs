use ratatui::{
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem},
    Frame,
};
use serde::{Deserialize, Serialize};

/// Watchlist entry from students
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatchlistEntry {
    pub team: String,
    pub company: String,
    pub flag_count: u32,
}

pub struct WatchlistWidget<'a> {
    entries: &'a [WatchlistEntry],
}

impl<'a> WatchlistWidget<'a> {
    pub fn new(entries: &'a [WatchlistEntry]) -> Self {
        Self { entries }
    }

    pub fn render(&self, frame: &mut Frame, area: ratatui::layout::Rect) {
        let items: Vec<ListItem> = self
            .entries
            .iter()
            .map(|entry| {
                let color = if entry.flag_count >= 3 {
                    Color::Red
                } else {
                    Color::Yellow
                };

                let line = Line::from(vec![
                    Span::styled(&entry.team, Style::default().fg(Color::Cyan)),
                    Span::raw(": "),
                    Span::raw(&entry.company),
                    Span::raw(" "),
                    Span::styled(
                        format!("({} flags)", entry.flag_count),
                        Style::default().fg(color),
                    ),
                ]);

                ListItem::new(line)
            })
            .collect();

        let list = List::new(items).block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Watchlist (compacted) "),
        );

        frame.render_widget(list, area);
    }
}
