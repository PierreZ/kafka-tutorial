//! Header widget displaying global stats and progress distribution

use crate::state::{GroupState, TeamState};
use crate::NUM_TEAMS;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Data needed for rendering the header
pub struct HeaderData<'a> {
    pub teams: &'a HashMap<String, TeamState>,
    pub group_states: &'a HashMap<String, GroupState>,
    pub session_start: Instant,
    pub last_data_update: Option<Instant>,
}

impl<'a> HeaderData<'a> {
    /// Get count of active teams (with Active group state)
    pub fn active_count(&self) -> usize {
        self.group_states
            .values()
            .filter(|s| **s == GroupState::Active)
            .count()
    }

    /// Get session duration as formatted string
    pub fn session_duration(&self) -> String {
        let elapsed = self.session_start.elapsed();
        let mins = elapsed.as_secs() / 60;
        let hours = mins / 60;
        let mins = mins % 60;
        if hours > 0 {
            format!("{}h{:02}m", hours, mins)
        } else {
            format!("{}m", mins)
        }
    }

    /// Check if data is fresh (updated within last 5 seconds)
    pub fn is_live(&self) -> bool {
        self.last_data_update
            .map(|t| t.elapsed() < Duration::from_secs(5))
            .unwrap_or(false)
    }

    /// Get step completion distribution [step0_count, step1_count, step2_count, step3_count, step4_count]
    pub fn step_distribution(&self) -> [usize; 5] {
        let mut dist = [0usize; 5];
        for team in self.teams.values() {
            let steps = team.step_count();
            if steps <= 4 {
                dist[steps] += 1;
            }
        }
        dist
    }

    /// Get total actions across all teams
    pub fn total_actions(&self) -> u64 {
        self.teams.values().map(|t| t.action_count).sum()
    }
}

/// Render the header (2 lines)
pub fn render(frame: &mut Frame, area: Rect, data: &HeaderData) {
    // Split into 2 lines
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(1)])
        .split(area);

    render_line1(frame, chunks[0], data);
    render_line2(frame, chunks[1], data);
}

/// Line 1: Title | Active Count | Live Status | Duration | Total Actions
fn render_line1(frame: &mut Frame, area: Rect, data: &HeaderData) {
    let active = data.active_count();
    let total = NUM_TEAMS;
    let duration = data.session_duration();
    let total_actions = data.total_actions();

    let live_span = if data.is_live() {
        Span::styled(" LIVE ", Style::default().fg(Color::Black).bg(Color::Green))
    } else {
        Span::styled(" ... ", Style::default().fg(Color::Black).bg(Color::Yellow))
    };

    let line = Line::from(vec![
        Span::styled(
            " KAFKA LEADERBOARD ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
        Span::styled(
            format!("{}/{} Active", active, total),
            Style::default().fg(Color::White),
        ),
        Span::raw("  "),
        live_span,
        Span::raw("  "),
        Span::styled(format!("{}", duration), Style::default().fg(Color::DarkGray)),
        Span::raw("  "),
        Span::styled(
            format!("{} actions", total_actions),
            Style::default().fg(Color::White),
        ),
    ]);

    frame.render_widget(Paragraph::new(line), area);
}

/// Line 2: Progress distribution bars
fn render_line2(frame: &mut Frame, area: Rect, data: &HeaderData) {
    let dist = data.step_distribution();
    let total = NUM_TEAMS as f32;

    // Build progress bars for each step
    let mut spans: Vec<Span> = vec![Span::raw(" ")];

    let step_labels = ["Step0", "Step1", "Step2", "Step3", "Step4"];
    let step_colors = [
        Color::DarkGray,
        Color::LightBlue,
        Color::Cyan,
        Color::Yellow,
        Color::Green,
    ];

    for (i, &count) in dist.iter().enumerate() {
        if i > 0 {
            spans.push(Span::raw("  "));
        }

        // Only show non-zero steps in the progress bar display
        let bar_width: usize = 8;
        let filled = ((count as f32 / total) * bar_width as f32).round() as usize;
        let empty = bar_width.saturating_sub(filled);

        let bar = format!(
            "{}{}",
            "█".repeat(filled),
            "░".repeat(empty)
        );

        spans.push(Span::styled(
            format!("{}: ", step_labels[i]),
            Style::default().fg(Color::DarkGray),
        ));
        spans.push(Span::styled(bar, Style::default().fg(step_colors[i])));
        spans.push(Span::styled(
            format!(" {}", count),
            Style::default().fg(step_colors[i]),
        ));
    }

    let line = Line::from(spans);
    frame.render_widget(Paragraph::new(line), area);
}
