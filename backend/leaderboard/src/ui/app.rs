//! Main application state and event loop

use crate::config::Settings;
use crate::kafka::{admin, consumer as kafka_consumer};
use crate::state::{persistence, AchievementType, GroupState, TeamState};
use crate::ui::widgets::{detail_panel, footer, header, help_overlay, leaderboard_table};
use crate::validation::{self, SimpleValidationResult};
use crate::NUM_TEAMS;
use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use futures::StreamExt;
use kafka_common::messages::WatchlistEntry;
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    widgets::TableState,
    Frame, Terminal,
};
use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::message::Message;
use rdkafka::producer::FutureProducer;
use std::collections::HashMap;
use std::io;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{error, info, warn};

/// View mode for the application
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewMode {
    Normal,
    Help,
}

/// Shared application state
pub struct AppState {
    pub teams: HashMap<String, TeamState>,
    pub consumer_counts: HashMap<String, u32>,
    pub group_states: HashMap<String, GroupState>,
    pub celebrations: Vec<footer::Celebration>,
    pub session_start: Instant,
    pub last_data_update: Option<Instant>,
}

impl AppState {
    pub fn new() -> Self {
        let mut teams = HashMap::new();
        // Initialize all 15 teams
        for i in 1..=NUM_TEAMS {
            let team_name = format!("team-{}", i);
            teams.insert(team_name.clone(), TeamState::new(team_name));
        }

        Self {
            teams,
            consumer_counts: HashMap::new(),
            group_states: HashMap::new(),
            celebrations: Vec::new(),
            session_start: Instant::now(),
            last_data_update: None,
        }
    }

    /// Get active celebration (most recent that's still showing)
    pub fn active_celebration(&self) -> Option<&footer::Celebration> {
        self.celebrations.iter().rev().find(|c| c.is_active())
    }

    /// Add a celebration
    pub fn add_celebration(&mut self, team_name: String, achievement: AchievementType) {
        self.celebrations.push(footer::Celebration {
            team_name,
            achievement,
            unlocked_at: Instant::now(),
        });
        // Keep only recent celebrations
        self.celebrations
            .retain(|c| c.unlocked_at.elapsed() < Duration::from_secs(10));
    }
}

/// UI state for rendering
pub struct UiState {
    pub view_mode: ViewMode,
    pub selected_row: usize,
    pub table_state: TableState,
    pub sorted_teams: Vec<String>,
    pub show_detail_panel: bool,
    pub fullscreen_mode: bool,
}

impl UiState {
    pub fn new() -> Self {
        let mut table_state = TableState::default();
        table_state.select(Some(0));
        Self {
            view_mode: ViewMode::Normal,
            selected_row: 0,
            table_state,
            sorted_teams: Vec::new(),
            show_detail_panel: true, // Show by default
            fullscreen_mode: false,
        }
    }

    pub fn move_up(&mut self, count: usize) {
        if self.selected_row > 0 {
            self.selected_row = self.selected_row.saturating_sub(count);
            self.table_state.select(Some(self.selected_row));
        }
    }

    pub fn move_down(&mut self, max: usize, count: usize) {
        if self.selected_row < max.saturating_sub(1) {
            self.selected_row = (self.selected_row + count).min(max.saturating_sub(1));
            self.table_state.select(Some(self.selected_row));
        }
    }

    pub fn go_to_start(&mut self) {
        self.selected_row = 0;
        self.table_state.select(Some(0));
    }

    pub fn go_to_end(&mut self, max: usize) {
        self.selected_row = max.saturating_sub(1);
        self.table_state.select(Some(self.selected_row));
    }

    pub fn selected_team(&self) -> Option<&String> {
        self.sorted_teams.get(self.selected_row)
    }

    pub fn toggle_detail_panel(&mut self) {
        self.show_detail_panel = !self.show_detail_panel;
    }

    pub fn toggle_fullscreen(&mut self) {
        self.fullscreen_mode = !self.fullscreen_mode;
        if self.fullscreen_mode {
            self.show_detail_panel = false;
        }
    }
}

/// Run the application
pub async fn run(settings: Settings) -> Result<()> {
    info!("Starting leaderboard application");

    // Restore state from Kafka
    let restored = persistence::restore_states(
        &settings.kafka.brokers,
        &settings.kafka.username,
        &settings.kafka.password,
        &settings.kafka.security_protocol,
        &settings.kafka.sasl_mechanism,
        &settings.topics.scorer_state,
    )
    .await?;

    let state = Arc::new(RwLock::new(AppState::new()));

    // Merge restored state
    {
        let mut app_state = state.write().await;
        for (team_name, team_state) in restored {
            app_state.teams.insert(team_name, team_state);
        }
    }

    // Create Kafka producer for state persistence
    let producer: FutureProducer = kafka_common::kafka::new_sasl_config(
        &settings.kafka.brokers,
        &settings.kafka.username,
        &settings.kafka.password,
        &settings.kafka.security_protocol,
        &settings.kafka.sasl_mechanism,
    )
    .create()?;

    let producer = Arc::new(producer);

    // Spawn consumer tasks
    let settings_clone = settings.clone();
    let state_clone = state.clone();
    let producer_clone = producer.clone();
    tokio::spawn(async move {
        if let Err(e) = consume_actions(settings_clone, state_clone, producer_clone).await {
            error!("Actions consumer error: {:?}", e);
        }
    });

    let settings_clone = settings.clone();
    let state_clone = state.clone();
    let producer_clone = producer.clone();
    tokio::spawn(async move {
        if let Err(e) = consume_watchlist(settings_clone, state_clone, producer_clone).await {
            error!("Watchlist consumer error: {:?}", e);
        }
    });

    // Spawn admin monitoring task
    let settings_clone = settings.clone();
    let state_clone = state.clone();
    let producer_clone = producer.clone();
    tokio::spawn(async move {
        loop {
            if let Err(e) =
                monitor_consumer_groups(&settings_clone, &state_clone, &producer_clone).await
            {
                warn!("Admin monitoring error: {:?}", e);
            }
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    });

    // Run TUI
    run_tui(state, settings).await
}

/// Run in demo mode (no Kafka)
pub async fn run_demo() -> Result<()> {
    info!("Running in demo mode");

    let state = Arc::new(RwLock::new(AppState::new()));

    // Add some fake data
    {
        let mut app_state = state.write().await;

        // team-1: All achievements
        if let Some(team) = app_state.teams.get_mut("team-1") {
            team.unlock_achievement(AchievementType::Connected);
            team.unlock_achievement(AchievementType::FirstLoad);
            team.unlock_achievement(AchievementType::Scaled);
            team.unlock_achievement(AchievementType::WatchlistDone);
            team.unlock_achievement(AchievementType::FirstBlood);
            team.unlock_achievement(AchievementType::HighThroughput);
            team.unlock_achievement(AchievementType::CleanStreak);
            team.action_count = 150;
            team.watchlist_count = 5;
            team.current_lag = 0;
        }
        app_state.consumer_counts.insert("team-1".to_string(), 3);
        app_state
            .group_states
            .insert("team-1".to_string(), GroupState::Active);

        // team-2: 3 steps
        if let Some(team) = app_state.teams.get_mut("team-2") {
            team.unlock_achievement(AchievementType::Connected);
            team.unlock_achievement(AchievementType::FirstLoad);
            team.unlock_achievement(AchievementType::Scaled);
            team.unlock_achievement(AchievementType::HighThroughput);
            team.action_count = 120;
            team.current_lag = 15;
        }
        app_state.consumer_counts.insert("team-2".to_string(), 2);
        app_state
            .group_states
            .insert("team-2".to_string(), GroupState::Active);

        // team-3: 2 steps with errors
        if let Some(team) = app_state.teams.get_mut("team-3") {
            team.unlock_achievement(AchievementType::Connected);
            team.unlock_achievement(AchievementType::FirstLoad);
            team.record_error(AchievementType::ParseError);
            team.record_error(AchievementType::ParseError);
            team.record_error(AchievementType::ParseError);
            team.action_count = 45;
            team.current_lag = 89;
        }
        app_state.consumer_counts.insert("team-3".to_string(), 1);
        app_state
            .group_states
            .insert("team-3".to_string(), GroupState::Active);

        // team-4: Just connected
        if let Some(team) = app_state.teams.get_mut("team-4") {
            team.unlock_achievement(AchievementType::Connected);
            team.action_count = 5;
            team.current_lag = 45;
        }
        app_state.consumer_counts.insert("team-4".to_string(), 1);
        app_state
            .group_states
            .insert("team-4".to_string(), GroupState::Active);

        // Rest are disconnected
        for i in 5..=NUM_TEAMS {
            app_state
                .group_states
                .insert(format!("team-{}", i), GroupState::Empty);
        }
    }

    let demo_settings = Settings {
        kafka: crate::config::KafkaSettings {
            brokers: "localhost:9092".to_string(),
            username: "demo".to_string(),
            password: "demo".to_string(),
            consumer_group: "demo".to_string(),
            security_protocol: "PLAINTEXT".to_string(),
            sasl_mechanism: "PLAIN".to_string(),
        },
        topics: crate::config::TopicSettings {
            new_users: "new_users".to_string(),
            actions: "actions".to_string(),
            watchlist: "watchlist".to_string(),
            scorer_state: "scorer_state".to_string(),
        },
        scoring: crate::config::ScoringSettings::default(),
        achievements: crate::config::AchievementSettings::default(),
    };

    run_tui(state, demo_settings).await
}

/// Run the TUI
async fn run_tui(state: Arc<RwLock<AppState>>, settings: Settings) -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut ui_state = UiState::new();

    let result = run_app(&mut terminal, state, &mut ui_state, &settings).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    result
}

/// Main application loop
async fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    state: Arc<RwLock<AppState>>,
    ui_state: &mut UiState,
    settings: &Settings,
) -> Result<()> {
    loop {
        // Draw
        {
            let app_state = state.read().await;
            ui_state.sorted_teams = leaderboard_table::get_sorted_teams(&app_state.teams);
            terminal.draw(|f| draw_ui(f, &app_state, ui_state, settings))?;
        }

        // Handle events with timeout for refresh
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                let max_rows = ui_state.sorted_teams.len();

                match ui_state.view_mode {
                    ViewMode::Normal => match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => break,
                        KeyCode::Char('?') | KeyCode::Char('h') => {
                            ui_state.view_mode = ViewMode::Help;
                        }
                        KeyCode::Up | KeyCode::Char('k') => ui_state.move_up(1),
                        KeyCode::Down | KeyCode::Char('j') => ui_state.move_down(max_rows, 1),
                        KeyCode::PageUp => ui_state.move_up(5),
                        KeyCode::PageDown => ui_state.move_down(max_rows, 5),
                        KeyCode::Home | KeyCode::Char('g') => ui_state.go_to_start(),
                        KeyCode::End | KeyCode::Char('G') => ui_state.go_to_end(max_rows),
                        KeyCode::Enter | KeyCode::Char('d') => {
                            ui_state.toggle_detail_panel();
                        }
                        KeyCode::Char('f') => {
                            ui_state.toggle_fullscreen();
                        }
                        KeyCode::Char('[') | KeyCode::Char('<') => {
                            ui_state.move_up(1);
                        }
                        KeyCode::Char(']') | KeyCode::Char('>') => {
                            ui_state.move_down(max_rows, 1);
                        }
                        _ => {}
                    },
                    ViewMode::Help => match key.code {
                        KeyCode::Esc
                        | KeyCode::Char('q')
                        | KeyCode::Char('?')
                        | KeyCode::Backspace => {
                            ui_state.view_mode = ViewMode::Normal;
                        }
                        _ => {}
                    },
                }
            }
        }
    }

    Ok(())
}

/// Draw the UI
fn draw_ui(frame: &mut Frame, app_state: &AppState, ui_state: &mut UiState, settings: &Settings) {
    let area = frame.area();

    // Main layout: header (2 lines), content (flex), footer (1 line)
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // Header
            Constraint::Min(10),   // Content
            Constraint::Length(1), // Footer
        ])
        .split(area);

    // Header
    let header_data = header::HeaderData {
        teams: &app_state.teams,
        group_states: &app_state.group_states,
        session_start: app_state.session_start,
        last_data_update: app_state.last_data_update,
    };
    header::render(frame, main_chunks[0], &header_data);

    // Content area - split horizontally if detail panel is visible
    let show_detail = ui_state.show_detail_panel && !ui_state.fullscreen_mode;

    if show_detail {
        let content_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(60), // Table
                Constraint::Percentage(40), // Detail
            ])
            .split(main_chunks[1]);

        // Leaderboard table
        let celebration_team = app_state.active_celebration().map(|c| c.team_name.as_str());
        let table_data = leaderboard_table::LeaderboardTableData {
            teams: &app_state.teams,
            consumer_counts: &app_state.consumer_counts,
            selected_row: ui_state.selected_row,
            celebration_team,
        };
        leaderboard_table::render(
            frame,
            content_chunks[0],
            &table_data,
            &ui_state.sorted_teams,
            &mut ui_state.table_state,
        );

        // Detail panel
        if let Some(team_name) = ui_state.selected_team() {
            if let Some(team_state) = app_state.teams.get(team_name) {
                let consumer_count = app_state
                    .consumer_counts
                    .get(team_name)
                    .copied()
                    .unwrap_or(0);

                let detail_data = detail_panel::DetailPanelData {
                    team_state,
                    consumer_count,
                    achievement_settings: &settings.achievements,
                };
                detail_panel::render(frame, content_chunks[1], &detail_data);
            } else {
                detail_panel::render_empty(frame, content_chunks[1]);
            }
        } else {
            detail_panel::render_empty(frame, content_chunks[1]);
        }
    } else {
        // Full width table
        let celebration_team = app_state.active_celebration().map(|c| c.team_name.as_str());
        let table_data = leaderboard_table::LeaderboardTableData {
            teams: &app_state.teams,
            consumer_counts: &app_state.consumer_counts,
            selected_row: ui_state.selected_row,
            celebration_team,
        };
        leaderboard_table::render(
            frame,
            main_chunks[1],
            &table_data,
            &ui_state.sorted_teams,
            &mut ui_state.table_state,
        );
    }

    // Footer
    let footer_data = footer::FooterData {
        selected_team: ui_state.selected_team().map(|s| s.as_str()),
        show_detail_panel: ui_state.show_detail_panel,
        active_celebration: app_state.active_celebration(),
    };
    footer::render(frame, main_chunks[2], &footer_data);

    // Overlays
    if ui_state.view_mode == ViewMode::Help {
        help_overlay::render(frame);
    }
}

/// Consume from actions topic
async fn consume_actions(
    settings: Settings,
    state: Arc<RwLock<AppState>>,
    producer: Arc<FutureProducer>,
) -> Result<()> {
    let consumer: StreamConsumer = kafka_consumer::create_consumer(
        &settings.kafka.brokers,
        &settings.kafka.username,
        &settings.kafka.password,
        &settings.kafka.security_protocol,
        &settings.kafka.sasl_mechanism,
        &format!("{}-actions", settings.kafka.consumer_group),
    )?;

    consumer.subscribe(&[&settings.topics.actions])?;
    info!("Subscribed to actions topic: {}", settings.topics.actions);

    let mut stream = consumer.stream();

    while let Some(result) = stream.next().await {
        match result {
            Ok(message) => {
                if let Some(payload) = message.payload() {
                    let validation_result = validation::validate_action_simple(payload);

                    let mut app_state = state.write().await;
                    app_state.last_data_update = Some(Instant::now());

                    match validation_result {
                        SimpleValidationResult::Valid { team: team_name } => {
                            // Clone values we need before mutable borrow
                            let thresholds = (
                                settings.achievements.high_throughput_threshold,
                                settings.achievements.clean_streak_threshold,
                            );
                            let scorer_topic = settings.topics.scorer_state.clone();

                            if let Some(team) = app_state.teams.get_mut(&team_name) {
                                team.action_count += 1;
                                team.last_action_time = Some(Instant::now());
                                team.recent_actions.push_back(Instant::now());
                                team.increment_clean_streak();

                                let action_count = team.action_count;
                                let clean_streak = team.clean_streak_count;

                                // Check achievements
                                let first_load =
                                    team.unlock_achievement(AchievementType::FirstLoad);
                                let high_throughput = action_count >= thresholds.0
                                    && team.unlock_achievement(AchievementType::HighThroughput);
                                let clean_streak_ach = clean_streak >= thresholds.1
                                    && team.unlock_achievement(AchievementType::CleanStreak);

                                // Persist if needed
                                if first_load || high_throughput || clean_streak_ach {
                                    let team_clone = team.clone();
                                    drop(app_state);

                                    let _ = persistence::persist_state(
                                        &producer,
                                        &scorer_topic,
                                        &team_clone,
                                    )
                                    .await;

                                    let mut app_state = state.write().await;
                                    if first_load {
                                        app_state.add_celebration(
                                            team_name.clone(),
                                            AchievementType::FirstLoad,
                                        );
                                    }
                                    if high_throughput {
                                        app_state.add_celebration(
                                            team_name.clone(),
                                            AchievementType::HighThroughput,
                                        );
                                    }
                                    if clean_streak_ach {
                                        app_state.add_celebration(
                                            team_name.clone(),
                                            AchievementType::CleanStreak,
                                        );
                                    }
                                }
                            }
                        }
                        SimpleValidationResult::InvalidJson => {
                            // Try to extract team name from raw JSON
                            if let Some(team_name) = validation::extract_team_from_payload(payload)
                            {
                                if let Some(team) = app_state.teams.get_mut(&team_name) {
                                    team.record_error(AchievementType::ParseError);
                                }
                            }
                        }
                        SimpleValidationResult::MissingFields { team: team_name } => {
                            if let Some(team) = app_state.teams.get_mut(&team_name) {
                                team.record_error(AchievementType::MissingFields);
                            }
                        }
                    }
                }
            }
            Err(e) => {
                warn!("Error consuming actions: {:?}", e);
            }
        }
    }

    Ok(())
}

/// Consume from watchlist topic
async fn consume_watchlist(
    settings: Settings,
    state: Arc<RwLock<AppState>>,
    producer: Arc<FutureProducer>,
) -> Result<()> {
    let consumer: StreamConsumer = kafka_consumer::create_consumer(
        &settings.kafka.brokers,
        &settings.kafka.username,
        &settings.kafka.password,
        &settings.kafka.security_protocol,
        &settings.kafka.sasl_mechanism,
        &format!("{}-watchlist", settings.kafka.consumer_group),
    )?;

    consumer.subscribe(&[&settings.topics.watchlist])?;
    info!(
        "Subscribed to watchlist topic: {}",
        settings.topics.watchlist
    );

    let mut stream = consumer.stream();

    while let Some(result) = stream.next().await {
        match result {
            Ok(message) => {
                if let Some(payload) = message.payload() {
                    if let Ok(entry) = serde_json::from_slice::<WatchlistEntry>(payload) {
                        let scorer_topic = settings.topics.scorer_state.clone();
                        let team_name = entry.team.clone();

                        let mut app_state = state.write().await;
                        app_state.last_data_update = Some(Instant::now());

                        let should_celebrate =
                            if let Some(team) = app_state.teams.get_mut(&team_name) {
                                team.watchlist_count += 1;
                                team.last_watchlist_time = Some(Instant::now());
                                team.unlock_achievement(AchievementType::WatchlistDone)
                            } else {
                                false
                            };

                        if should_celebrate {
                            if let Some(team) = app_state.teams.get(&team_name) {
                                let team_clone = team.clone();
                                drop(app_state);

                                let _ = persistence::persist_state(
                                    &producer,
                                    &scorer_topic,
                                    &team_clone,
                                )
                                .await;

                                let mut app_state = state.write().await;
                                app_state.add_celebration(
                                    team_name.clone(),
                                    AchievementType::WatchlistDone,
                                );
                            }
                        }
                    }
                }
            }
            Err(e) => {
                warn!("Error consuming watchlist: {:?}", e);
            }
        }
    }

    Ok(())
}

/// Monitor consumer groups for connection and scaling achievements
async fn monitor_consumer_groups(
    settings: &Settings,
    state: &Arc<RwLock<AppState>>,
    producer: &Arc<FutureProducer>,
) -> Result<()> {
    // Fetch all consumer group statuses at once
    let statuses = admin::fetch_consumer_group_statuses(
        &settings.kafka.brokers,
        &settings.kafka.username,
        &settings.kafka.password,
        &settings.kafka.security_protocol,
        &settings.kafka.sasl_mechanism,
    )
    .await;

    // Check first blood status
    let first_blood_awarded = {
        let app_state = state.read().await;
        app_state
            .teams
            .values()
            .any(|t| t.has_achievement(AchievementType::FirstBlood))
    };

    let mut first_blood_given = first_blood_awarded;

    for status in statuses {
        let team_name = status.team_name.clone();

        let mut app_state = state.write().await;
        app_state
            .consumer_counts
            .insert(team_name.clone(), status.members);
        app_state
            .group_states
            .insert(team_name.clone(), status.state.clone());
        app_state.last_data_update = Some(Instant::now());

        // Collect achievements to celebrate (avoid borrow issues)
        let mut celebrations = Vec::new();
        let mut should_persist = false;

        if let Some(team) = app_state.teams.get_mut(&team_name) {
            // Fetch actual lag from consumer group committed offsets
            if let Some(lag) = admin::fetch_consumer_group_lag(
                &settings.kafka.brokers,
                &settings.kafka.username,
                &settings.kafka.password,
                &settings.kafka.security_protocol,
                &settings.kafka.sasl_mechanism,
                &team_name,
                &settings.topics.new_users,
            ) {
                team.update_lag(lag);
            }

            // Check Connected achievement
            if status.state == GroupState::Active
                && team.unlock_achievement(AchievementType::Connected)
            {
                celebrations.push(AchievementType::Connected);
                should_persist = true;

                // Check First Blood
                if !first_blood_given {
                    team.unlock_achievement(AchievementType::FirstBlood);
                    celebrations.push(AchievementType::FirstBlood);
                    first_blood_given = true;
                }
            }

            // Check Scaled achievement
            if status.members >= settings.achievements.scaled_members
                && team.unlock_achievement(AchievementType::Scaled)
            {
                celebrations.push(AchievementType::Scaled);
                should_persist = true;
            }

            // Check Partition Explorer achievement
            if status.members >= settings.achievements.partition_explorer_members
                && team.unlock_achievement(AchievementType::PartitionExplorer)
            {
                celebrations.push(AchievementType::PartitionExplorer);
            }

            // Check Lag Buster achievement
            if team.qualifies_for_lag_buster(settings.achievements.lag_buster_threshold)
                && team.unlock_achievement(AchievementType::LagBuster)
            {
                celebrations.push(AchievementType::LagBuster);
            }

            // Check Champion achievement
            if team.has_all_champion_requirements()
                && team.unlock_achievement(AchievementType::Champion)
            {
                celebrations.push(AchievementType::Champion);
            }
        }

        // Add celebrations
        for achievement in &celebrations {
            app_state.add_celebration(team_name.clone(), *achievement);
        }

        // Persist if needed
        if should_persist {
            if let Some(team) = app_state.teams.get(&team_name) {
                let team_clone = team.clone();
                let scorer_topic = settings.topics.scorer_state.clone();
                drop(app_state);

                let _ = persistence::persist_state(producer, &scorer_topic, &team_clone).await;
            }
        }
    }

    Ok(())
}
