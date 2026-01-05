//! Footer widget - keybindings and celebration announcements

use crate::state::AchievementType;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};
use std::time::Instant;

/// Celebration data for achievement unlocks
#[derive(Debug, Clone)]
pub struct Celebration {
    pub team_name: String,
    pub achievement: AchievementType,
    pub unlocked_at: Instant,
}

impl Celebration {
    /// Check if celebration is still active (3 second display)
    pub fn is_active(&self) -> bool {
        self.unlocked_at.elapsed().as_secs() < 3
    }
}

/// Data needed for rendering the footer
pub struct FooterData<'a> {
    pub selected_team: Option<&'a str>,
    pub show_detail_panel: bool,
    pub active_celebration: Option<&'a Celebration>,
}

/// Render the footer
pub fn render(frame: &mut Frame, area: Rect, data: &FooterData) {
    // Check for celebration first
    if let Some(celebration) = data.active_celebration {
        if celebration.is_active() {
            render_celebration(frame, area, celebration);
            return;
        }
    }

    // Normal footer with keybindings
    render_keybindings(frame, area, data);
}

/// Render celebration announcement
fn render_celebration(frame: &mut Frame, area: Rect, celebration: &Celebration) {
    let line = Line::from(vec![
        Span::styled(
            " ACHIEVEMENT! ",
            Style::default()
                .fg(Color::Black)
                .bg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" "),
        Span::styled(
            &celebration.team_name,
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" earned "),
        Span::styled(
            format!(
                "{} {}",
                celebration.achievement.emoji(),
                celebration.achievement.name()
            ),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("!"),
    ]);

    frame.render_widget(Paragraph::new(line), area);
}

/// Render keybindings footer
fn render_keybindings(frame: &mut Frame, area: Rect, data: &FooterData) {
    let detail_key = if data.show_detail_panel {
        "[d] Hide"
    } else {
        "[d] Detail"
    };

    let mut spans = vec![
        Span::styled(" [", Style::default().fg(Color::DarkGray)),
        Span::styled("jk", Style::default().fg(Color::White)),
        Span::styled("] Move ", Style::default().fg(Color::DarkGray)),
        Span::styled(detail_key, Style::default().fg(Color::DarkGray)),
        Span::styled(" [f] Full ", Style::default().fg(Color::DarkGray)),
        Span::styled("[?] Help ", Style::default().fg(Color::DarkGray)),
        Span::styled("[q] Quit", Style::default().fg(Color::DarkGray)),
    ];

    // Add selected team info
    if let Some(team) = data.selected_team {
        spans.push(Span::raw("  "));
        spans.push(Span::styled(team, Style::default().fg(Color::White)));
    }

    let line = Line::from(spans);
    frame.render_widget(Paragraph::new(line), area);
}
