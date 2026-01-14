//! Detail panel widget - side panel showing selected team details

use crate::config::AchievementSettings;
use crate::state::{AchievementType, TeamState, STEP_INCOMPLETE};
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

/// Data needed for rendering the detail panel
pub struct DetailPanelData<'a> {
    pub team_state: &'a TeamState,
    pub consumer_count: u32,
    pub achievement_settings: &'a AchievementSettings,
}

/// Render the detail panel
pub fn render(frame: &mut Frame, area: Rect, data: &DetailPanelData) {
    let mut lines: Vec<Line> = Vec::new();

    // Team header
    lines.push(Line::from(vec![Span::styled(
        format!(" {} ", data.team_state.team_name.to_uppercase()),
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    )]));
    lines.push(Line::from(""));

    // Step Progress section
    lines.push(Line::from(vec![Span::styled(
        "Progress",
        Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD),
    )]));

    for step in AchievementType::all_steps() {
        let has_it = data.team_state.has_achievement(step);
        let emoji = if has_it {
            step.emoji()
        } else {
            STEP_INCOMPLETE
        };
        let status = if has_it { "OK" } else { "  " };

        let time_info = if has_it {
            data.team_state
                .achievement_timestamps
                .get(&step)
                .map(|t| {
                    let duration = chrono::Utc::now().signed_duration_since(*t);
                    let mins = duration.num_minutes();
                    if mins < 60 {
                        format!("{}m", mins)
                    } else {
                        format!("{}h", mins / 60)
                    }
                })
                .unwrap_or_default()
        } else {
            "".to_string()
        };

        let style = if has_it {
            Style::default().fg(Color::Green)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        lines.push(Line::from(vec![
            Span::raw(" "),
            Span::styled(format!("{} {:<12}", emoji, step.name()), style),
            Span::styled(format!("{:>2}", status), style),
            Span::raw(" "),
            Span::styled(time_info, Style::default().fg(Color::DarkGray)),
        ]));
    }

    lines.push(Line::from(""));

    // Bonus Achievements section
    lines.push(Line::from(vec![Span::styled(
        "Bonus",
        Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD),
    )]));

    for achievement in AchievementType::all_bonus() {
        let has_it = data.team_state.has_achievement(achievement);
        let emoji = achievement.emoji();

        // Progress info for countable achievements
        let progress_info = match achievement {
            AchievementType::HighThroughput => {
                let current = data.team_state.action_count;
                let target = data.achievement_settings.high_throughput_threshold;
                if has_it {
                    format!("{}", current)
                } else {
                    format!("{}/{}", current, target)
                }
            }
            AchievementType::CleanStreak => {
                let current = data.team_state.clean_streak_count;
                let target = data.achievement_settings.clean_streak_threshold;
                if has_it {
                    format!("{}", current)
                } else {
                    format!("{}/{}", current, target)
                }
            }
            AchievementType::LagBuster => {
                if has_it {
                    "OK".to_string()
                } else if data.team_state.max_lag_seen > 0 {
                    format!(
                        "max:{}/{}",
                        data.team_state.max_lag_seen,
                        data.achievement_settings.lag_buster_threshold
                    )
                } else {
                    "".to_string()
                }
            }
            AchievementType::PartitionExplorer => {
                if has_it {
                    format!("{}", data.consumer_count)
                } else {
                    format!(
                        "{}/{}",
                        data.consumer_count, data.achievement_settings.partition_explorer_members
                    )
                }
            }
            AchievementType::KeyMaster => {
                let current = data.team_state.correct_key_count;
                let target = 25u64;
                if has_it {
                    format!("{}", current)
                } else {
                    format!("{}/{}", current, target)
                }
            }
            _ => {
                if has_it {
                    "OK".to_string()
                } else {
                    "".to_string()
                }
            }
        };

        let style = if has_it {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        lines.push(Line::from(vec![
            Span::raw(" "),
            Span::styled(format!("{} {:<14}", emoji, achievement.name()), style),
            Span::styled(progress_info, Style::default().fg(Color::DarkGray)),
        ]));
    }

    lines.push(Line::from(""));

    // Stats section
    lines.push(Line::from(vec![Span::styled(
        "Stats",
        Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD),
    )]));

    let parse_errors = data.team_state.get_error_count(AchievementType::ParseError);
    let field_errors = data
        .team_state
        .get_error_count(AchievementType::MissingFields);
    let total_errors = parse_errors + field_errors;

    let stats = [
        ("Actions", format!("{}", data.team_state.action_count)),
        ("Stats", format!("{}", data.team_state.stats_count)),
        ("Keys OK", format!("{}", data.team_state.correct_key_count)),
        ("Consumers", format!("{}", data.consumer_count)),
        ("Lag", format!("{}", data.team_state.current_lag)),
        (
            "Errors",
            if total_errors > 0 {
                format!("{}", total_errors)
            } else {
                "-".to_string()
            },
        ),
        ("Session", data.team_state.session_duration_display()),
    ];

    for (label, value) in stats {
        lines.push(Line::from(vec![
            Span::raw(" "),
            Span::styled(
                format!("{:<10}", label),
                Style::default().fg(Color::DarkGray),
            ),
            Span::styled(value, Style::default().fg(Color::White)),
        ]));
    }

    // If there are errors, show breakdown
    if total_errors > 0 {
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::raw(" "),
            Span::styled(
                format!("  Parse: {} Field: {}", parse_errors, field_errors),
                Style::default().fg(Color::Red),
            ),
        ]));
    }

    let content = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray))
                .title(Line::from(vec![Span::styled(
                    " Details ",
                    Style::default().fg(Color::White),
                )])),
        )
        .wrap(Wrap { trim: false });

    frame.render_widget(content, area);
}

/// Render an empty detail panel with instructions
pub fn render_empty(frame: &mut Frame, area: Rect) {
    let lines = vec![
        Line::from(""),
        Line::from(""),
        Line::from(vec![Span::styled(
            "  Select a team",
            Style::default().fg(Color::DarkGray),
        )]),
        Line::from(vec![Span::styled(
            "  to view details",
            Style::default().fg(Color::DarkGray),
        )]),
    ];

    let content = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray))
            .title(Line::from(vec![Span::styled(
                " Details ",
                Style::default().fg(Color::White),
            )])),
    );

    frame.render_widget(content, area);
}
