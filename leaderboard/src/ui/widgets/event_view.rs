use crate::ui::event_buffer::EventBuffer;
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem},
    Frame,
};

pub struct EventViewWidget<'a> {
    buffer: &'a EventBuffer,
    title: &'a str,
    scroll_offset: usize,
}

impl<'a> EventViewWidget<'a> {
    pub fn new(buffer: &'a EventBuffer, title: &'a str, scroll_offset: usize) -> Self {
        Self {
            buffer,
            title,
            scroll_offset,
        }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        // Calculate visible lines (area height minus borders)
        let visible_lines = (area.height as usize).saturating_sub(2);

        // Build list items (newest first, then skip/take for scrolling)
        let items: Vec<ListItem> = self
            .buffer
            .iter_rev()
            .skip(self.scroll_offset)
            .take(visible_lines)
            .map(|event| self.render_event(event, area.width as usize))
            .collect::<Vec<_>>()
            .into_iter()
            .rev() // Reverse back so newest is at bottom
            .collect();

        let list = List::new(items).block(Block::default().borders(Borders::ALL).title(format!(
            " {} ({} events) ",
            self.title,
            self.buffer.len()
        )));

        frame.render_widget(list, area);
    }

    fn render_event(
        &self,
        event: &crate::ui::event_buffer::BufferedEvent,
        max_width: usize,
    ) -> ListItem<'static> {
        let time_str = event.relative_time();
        let time_width = 5; // "999s " or " 5m+ " - fixed width for alignment

        let mut spans = vec![Span::styled(
            format!("{:>4} ", time_str),
            Style::default().fg(Color::Gray),
        )];

        // Add team badge if present
        let badge_width = if let Some(ref team) = event.team {
            let badge = format!("[{}] ", team);
            let width = badge.len();
            spans.push(Span::raw(badge));
            width
        } else {
            0
        };

        // Calculate remaining width for JSON (account for borders)
        let json_max = max_width.saturating_sub(time_width + badge_width + 2);

        // Truncate JSON if needed
        let json_display = if event.json.len() > json_max {
            format!("{}...", &event.json[..json_max.saturating_sub(3)])
        } else {
            event.json.clone()
        };

        spans.push(Span::raw(json_display));

        ListItem::new(Line::from(spans))
    }
}
