use crate::ui::event_buffer::{EventBuffer, ParsedEventData};
use ratatui::{
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Row, Table},
    Frame,
};

/// Table widget for displaying actions with validation status
pub struct ActionsTableWidget<'a> {
    buffer: &'a EventBuffer,
    scroll_offset: usize,
}

impl<'a> ActionsTableWidget<'a> {
    pub fn new(buffer: &'a EventBuffer, scroll_offset: usize) -> Self {
        Self {
            buffer,
            scroll_offset,
        }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let visible_rows = (area.height as usize).saturating_sub(4); // header + borders + title

        let header = Row::new(vec![
            Cell::from("Time"),
            Cell::from("Team"),
            Cell::from("Customer"),
            Cell::from("Type"),
            Cell::from("Reason"),
            Cell::from("Status"),
        ])
        .style(
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(Color::Cyan),
        )
        .height(1);

        let rows: Vec<Row> = self
            .buffer
            .iter_rev()
            .skip(self.scroll_offset)
            .take(visible_rows)
            .map(|event| {
                let time = event.relative_time();

                match &event.parsed {
                    Some(ParsedEventData::Action { action, is_valid }) => {
                        let customer = action.customer.as_deref().unwrap_or("-");
                        let action_type = action.action_type.as_deref().unwrap_or("-");
                        let reason = action.reason.as_deref().unwrap_or("-");

                        // Truncate long fields
                        let customer_display = truncate(customer, 22);
                        let reason_display = truncate(reason, 20);

                        let (status, row_style) = if *is_valid {
                            ("OK", Style::default())
                        } else {
                            ("ERR", Style::default().bg(Color::Red).fg(Color::White))
                        };

                        Row::new(vec![
                            Cell::from(format!("{:>4}", time)),
                            Cell::from(action.team.clone()),
                            Cell::from(customer_display),
                            Cell::from(action_type.to_string()),
                            Cell::from(reason_display),
                            Cell::from(status),
                        ])
                        .style(row_style)
                    }
                    Some(ParsedEventData::Invalid) => {
                        // Invalid JSON - show error row
                        let team = event.team.clone().unwrap_or_else(|| "-".to_string());
                        Row::new(vec![
                            Cell::from(format!("{:>4}", time)),
                            Cell::from(team),
                            Cell::from("-"),
                            Cell::from("-"),
                            Cell::from("Parse error"),
                            Cell::from("ERR"),
                        ])
                        .style(Style::default().bg(Color::Red).fg(Color::White))
                    }
                    _ => {
                        // Fallback for legacy events without parsed data
                        Row::new(vec![
                            Cell::from(format!("{:>4}", time)),
                            Cell::from(event.team.clone().unwrap_or_else(|| "-".to_string())),
                            Cell::from("-"),
                            Cell::from("-"),
                            Cell::from("-"),
                            Cell::from("?"),
                        ])
                    }
                }
            })
            .collect::<Vec<_>>()
            .into_iter()
            .rev() // Reverse so newest is at bottom
            .collect();

        let widths = [
            Constraint::Length(5),  // Time
            Constraint::Length(10), // Team
            Constraint::Length(24), // Customer
            Constraint::Length(9),  // Type
            Constraint::Length(22), // Reason
            Constraint::Length(6),  // Status
        ];

        let title = format!(" actions ({} events) ", self.buffer.len());

        let table = Table::new(rows, widths)
            .header(header)
            .block(Block::default().borders(Borders::ALL).title(title));

        frame.render_widget(table, area);
    }
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() > max_len {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    } else {
        s.to_string()
    }
}
