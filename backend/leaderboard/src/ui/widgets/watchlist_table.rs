use crate::ui::event_buffer::{EventBuffer, ParsedEventData};
use ratatui::{
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Row, Table},
    Frame,
};

/// Table widget for displaying watchlist entries
pub struct WatchlistTableWidget<'a> {
    buffer: &'a EventBuffer,
    scroll_offset: usize,
}

impl<'a> WatchlistTableWidget<'a> {
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
            Cell::from("Company"),
            Cell::from("Flags"),
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
                    Some(ParsedEventData::Watchlist(entry)) => {
                        // Color based on flag_count
                        let count_style = if entry.flag_count >= 5 {
                            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
                        } else if entry.flag_count >= 3 {
                            Style::default().fg(Color::Yellow)
                        } else {
                            Style::default().fg(Color::Green)
                        };

                        // Truncate company name if too long
                        let company_display = if entry.company.len() > 35 {
                            format!("{}...", &entry.company[..32])
                        } else {
                            entry.company.clone()
                        };

                        Row::new(vec![
                            Cell::from(format!("{:>4}", time)),
                            Cell::from(entry.team.clone()),
                            Cell::from(company_display),
                            Cell::from(format!("{}", entry.flag_count)).style(count_style),
                        ])
                    }
                    _ => {
                        // Fallback for legacy/invalid events
                        Row::new(vec![
                            Cell::from(format!("{:>4}", time)),
                            Cell::from(event.team.clone().unwrap_or_else(|| "-".to_string())),
                            Cell::from("-"),
                            Cell::from("-"),
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
            Constraint::Length(38), // Company
            Constraint::Length(6),  // Flags
        ];

        let title = format!(" watchlist ({} events) ", self.buffer.len());

        let table = Table::new(rows, widths)
            .header(header)
            .block(Block::default().borders(Borders::ALL).title(title));

        frame.render_widget(table, area);
    }
}
