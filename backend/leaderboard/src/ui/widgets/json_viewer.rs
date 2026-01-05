use crate::ui::event_buffer::{EventBuffer, ParsedEventData};
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState},
    Frame,
};

/// Widget for displaying a single message with pretty-printed JSON
pub struct JsonViewerWidget<'a> {
    buffer: &'a EventBuffer,
    selected_index: usize, // 0 = newest
    scroll_offset: usize,  // vertical scroll within message
}

impl<'a> JsonViewerWidget<'a> {
    pub fn new(buffer: &'a EventBuffer, selected_index: usize, scroll_offset: usize) -> Self {
        Self {
            buffer,
            selected_index,
            scroll_offset,
        }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let total = self.buffer.len();
        let message_num = total.saturating_sub(self.selected_index);

        // Get the selected event
        let event = self.buffer.get_rev(self.selected_index);

        let (title, content) = match event {
            Some(e) => {
                // Get pretty JSON from parsed data or fallback
                let json = match &e.parsed {
                    Some(ParsedEventData::NewUser { pretty_json }) => pretty_json.clone(),
                    _ => {
                        // Fallback: pretty-print compact JSON
                        serde_json::from_str::<serde_json::Value>(&e.json)
                            .map(|v| {
                                serde_json::to_string_pretty(&v).unwrap_or_else(|_| e.json.clone())
                            })
                            .unwrap_or_else(|_| e.json.clone())
                    }
                };
                (
                    format!(
                        " new_users | Message {} of {} | {} ",
                        message_num, total, e.relative_time()
                    ),
                    json,
                )
            }
            None => (
                " new_users | No messages ".to_string(),
                "Waiting for messages from the producer...\n\nOnce messages arrive, use:\n  Up/Down: Navigate between messages\n  PgUp/PgDn: Scroll within JSON".to_string(),
            ),
        };

        // Split content into lines for scrolling
        let all_lines: Vec<&str> = content.lines().collect();
        let total_lines = all_lines.len();
        let visible_height = (area.height as usize).saturating_sub(4); // borders + hint line

        // Clamp scroll offset
        let max_scroll = total_lines.saturating_sub(visible_height);
        let scroll = self.scroll_offset.min(max_scroll);

        let mut lines: Vec<Line> = all_lines
            .iter()
            .skip(scroll)
            .take(visible_height)
            .map(|s| Line::from(s.to_string()))
            .collect();

        // Add empty lines if content is shorter than viewport
        while lines.len() < visible_height {
            lines.push(Line::from(""));
        }

        // Add navigation hint at bottom
        let hint = Line::from(vec![
            Span::styled(" Up/Down", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(": prev/next msg | "),
            Span::styled("PgUp/PgDn", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(": scroll | "),
            Span::styled("Home/End", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(": jump"),
        ])
        .style(Style::default().fg(Color::DarkGray));
        lines.push(hint);

        let paragraph = Paragraph::new(lines)
            .block(Block::default().borders(Borders::ALL).title(title))
            .style(Style::default().fg(Color::White));

        frame.render_widget(paragraph, area);

        // Render scrollbar if content overflows
        if total_lines > visible_height {
            let scrollbar = Scrollbar::default().orientation(ScrollbarOrientation::VerticalRight);
            let mut scrollbar_state = ScrollbarState::new(total_lines).position(scroll);
            let scrollbar_area = Rect {
                x: area.x + area.width - 2,
                y: area.y + 1,
                width: 1,
                height: area.height.saturating_sub(3),
            };
            frame.render_stateful_widget(scrollbar, scrollbar_area, &mut scrollbar_state);
        }
    }
}
