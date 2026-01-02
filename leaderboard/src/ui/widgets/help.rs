use crate::state::achievements::AchievementType;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

pub struct HelpWidget;

impl HelpWidget {
    pub fn render(frame: &mut Frame, area: Rect) {
        // Create centered popup area (70% width, 80% height)
        let popup_area = centered_rect(70, 80, area);

        // Clear background
        frame.render_widget(Clear, popup_area);

        let help_content = Self::build_help_content();

        let help_block = Paragraph::new(help_content)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Help - Press 'h' or Esc to close ")
                    .title_style(Style::default().add_modifier(Modifier::BOLD))
                    .style(Style::default().bg(Color::Black)),
            )
            .wrap(Wrap { trim: true });

        frame.render_widget(help_block, popup_area);
    }

    fn build_help_content() -> Vec<Line<'static>> {
        let mut lines = vec![];

        // Section: Step Achievements
        lines.push(Line::from(Span::styled(
            "=== STEP ACHIEVEMENTS (Required) ===",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(""));

        for step in AchievementType::all_steps() {
            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled(step.emoji(), Style::default().fg(Color::Green)),
                Span::raw("  "),
                Span::styled(step.name(), Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" - "),
                Span::raw(step.description()),
            ]));
        }
        lines.push(Line::from("  ⬜  Not yet earned"));
        lines.push(Line::from(""));

        // Section: Error Indicators
        lines.push(Line::from(Span::styled(
            "=== ERROR INDICATORS ===",
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(""));

        for error in AchievementType::all_errors() {
            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled(error.emoji(), Style::default().fg(Color::Red)),
                Span::raw("  "),
                Span::styled(error.name(), Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" - "),
                Span::raw(error.description()),
            ]));
        }
        lines.push(Line::from(""));

        // Section: Bonus Achievements
        lines.push(Line::from(Span::styled(
            "=== BONUS ACHIEVEMENTS ===",
            Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(""));

        for bonus in AchievementType::all_bonus() {
            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled(bonus.emoji(), Style::default().fg(Color::Magenta)),
                Span::raw("  "),
                Span::styled(bonus.name(), Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" - "),
                Span::raw(bonus.description()),
            ]));
        }
        lines.push(Line::from(""));

        // Section: Tab Navigation
        lines.push(Line::from(Span::styled(
            "=== TAB NAVIGATION ===",
            Style::default()
                .fg(Color::Blue)
                .add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(""));
        lines.push(Line::from("  1           Leaderboard view"));
        lines.push(Line::from("  2           new_users events"));
        lines.push(Line::from("  3           actions events"));
        lines.push(Line::from("  4           watchlist events"));
        lines.push(Line::from(""));

        // Section: Keyboard Shortcuts
        lines.push(Line::from(Span::styled(
            "=== KEYBOARD SHORTCUTS ===",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(""));
        lines.push(Line::from("  ↑/k         Move selection up / Scroll older"));
        lines.push(Line::from(
            "  ↓/j         Move selection down / Scroll newer",
        ));
        lines.push(Line::from("  Enter       View team details (leaderboard)"));
        lines.push(Line::from("  Home        Jump to first / newest events"));
        lines.push(Line::from("  End         Jump to last / oldest events"));
        lines.push(Line::from("  h / ?       Show this help screen"));
        lines.push(Line::from("  q / Esc     Quit (or close popup)"));
        lines.push(Line::from(""));

        // Section: Column Legend
        lines.push(Line::from(Span::styled(
            "=== COLUMN LEGEND ===",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(
            "  Team        - Team name (team-1 through team-15)",
        ));
        lines.push(Line::from("  Achievements- Step and bonus badges earned"));
        lines.push(Line::from("  Errors      - Error indicators with counts"));
        lines.push(Line::from("  👥          - Consumer group member count"));
        lines.push(Line::from("  📤          - Valid action count"));

        lines
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_width = r.width * percent_x / 100;
    let popup_height = r.height * percent_y / 100;
    let popup_x = (r.width - popup_width) / 2;
    let popup_y = (r.height - popup_height) / 2;

    Rect::new(r.x + popup_x, r.y + popup_y, popup_width, popup_height)
}
