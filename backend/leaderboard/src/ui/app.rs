//! Main application state and event loop

use crate::config::Settings;
use crate::kafka::{admin, consumer as kafka_consumer};
use crate::state::team::TeamStatsSnapshot;
use crate::state::{persistence, AchievementType, GroupState, TeamState};
use crate::ui::widgets::{
    detail_panel, footer, header, help_overlay, leaderboard_table, stats_panel,
};
use crate::validation::{self, SimpleValidationResult};
use crate::NUM_TEAMS;
use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use futures::StreamExt;
use kafka_common::messages::TeamStats;
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
        if let Err(e) = consume_team_stats(settings_clone, state_clone, producer_clone).await {
            error!("Team stats consumer error: {:?}", e);
        }
    });

    // Spawn per-team monitoring tasks (one independent task per team)
    for team_num in 1..=NUM_TEAMS {
        let team_name = format!("team-{}", team_num);
        let settings_clone = settings.clone();
        let state_clone = state.clone();
        let producer_clone = producer.clone();

        tokio::spawn(async move {
            monitor_single_team(team_name, settings_clone, state_clone, producer_clone).await;
        });
    }

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

        // team-1: All step achievements
        if let Some(team) = app_state.teams.get_mut("team-1") {
            team.unlock_achievement(AchievementType::Connected);
            team.unlock_achievement(AchievementType::FirstLoad);
            team.unlock_achievement(AchievementType::Scaled);
            team.unlock_achievement(AchievementType::StatsDone);
            team.action_count = 150;
            team.stats_count = 35;
            team.current_lag = 0;
            team.latest_stats = Some(TeamStatsSnapshot {
                processed: 500,
                flagged: 150,
            });
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
            team.action_count = 120;
            team.stats_count = 18;
            team.current_lag = 15;
            team.latest_stats = Some(TeamStatsSnapshot {
                processed: 320,
                flagged: 120,
            });
        }
        app_state.consumer_counts.insert("team-2".to_string(), 2);
        app_state
            .group_states
            .insert("team-2".to_string(), GroupState::Active);

        // team-3: 2 steps
        if let Some(team) = app_state.teams.get_mut("team-3") {
            team.unlock_achievement(AchievementType::Connected);
            team.unlock_achievement(AchievementType::FirstLoad);
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
            team_stats: "team_stats".to_string(),
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
    // Cache state snapshot for non-blocking rendering
    let mut cached_state: Option<AppState> = None;

    loop {
        // Try to get latest state without blocking (non-blocking for smooth UI)
        if let Ok(app_state) = state.try_read() {
            ui_state.sorted_teams = leaderboard_table::get_sorted_teams(&app_state.teams);
            cached_state = Some(AppState {
                teams: app_state.teams.clone(),
                consumer_counts: app_state.consumer_counts.clone(),
                group_states: app_state.group_states.clone(),
                celebrations: app_state.celebrations.clone(),
                session_start: app_state.session_start,
                last_data_update: app_state.last_data_update,
            });
        }

        // Draw using cached state
        if let Some(ref app_state) = cached_state {
            terminal.draw(|f| draw_ui(f, app_state, ui_state, settings))?;
        }

        // Handle events with timeout for refresh (~60 FPS)
        if event::poll(Duration::from_millis(16))? {
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
fn draw_ui(frame: &mut Frame, app_state: &AppState, ui_state: &mut UiState, _settings: &Settings) {
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
        // Three-panel layout: Table (50%) + Detail (25%) + Stats (25%)
        let content_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(50), // Table
                Constraint::Percentage(25), // Detail
                Constraint::Percentage(25), // Stats
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
                };
                detail_panel::render(frame, content_chunks[1], &detail_data);
            } else {
                detail_panel::render_empty(frame, content_chunks[1]);
            }
        } else {
            detail_panel::render_empty(frame, content_chunks[1]);
        }

        // Stats panel - shows live stats from compacted topic
        let stats_data = stats_panel::StatsPanelData {
            teams: &app_state.teams,
            sorted_teams: &ui_state.sorted_teams,
        };
        stats_panel::render(frame, content_chunks[2], &stats_data);
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
                            let scorer_topic = settings.topics.scorer_state.clone();

                            if let Some(team) = app_state.teams.get_mut(&team_name) {
                                team.action_count += 1;
                                team.last_action_time = Some(Instant::now());
                                team.recent_actions.push_back(Instant::now());

                                // Check FirstLoad achievement
                                let first_load =
                                    team.unlock_achievement(AchievementType::FirstLoad);

                                // Persist if needed
                                if first_load {
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
                                        AchievementType::FirstLoad,
                                    );
                                }
                            }
                        }
                        SimpleValidationResult::InvalidJson => {
                            // Just log, no error tracking
                        }
                        SimpleValidationResult::MissingFields { team: _team_name } => {
                            // Just log, no error tracking
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

/// Consume from team_stats topic
async fn consume_team_stats(
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
        &format!("{}-team-stats", settings.kafka.consumer_group),
    )?;

    consumer.subscribe(&[&settings.topics.team_stats])?;
    info!(
        "Subscribed to team_stats topic: {}",
        settings.topics.team_stats
    );

    let mut stream = consumer.stream();

    while let Some(result) = stream.next().await {
        match result {
            Ok(message) => {
                if let Some(payload) = message.payload() {
                    if let Ok(stats) = serde_json::from_slice::<TeamStats>(payload) {
                        let scorer_topic = settings.topics.scorer_state.clone();
                        let team_name = stats.team.clone();

                        let snapshot = TeamStatsSnapshot {
                            processed: stats.processed,
                            flagged: stats.flagged,
                        };

                        let mut app_state = state.write().await;
                        app_state.last_data_update = Some(Instant::now());

                        let mut celebrations = Vec::new();
                        let mut should_persist = false;

                        if let Some(team) = app_state.teams.get_mut(&team_name) {
                            team.record_stats_message(snapshot);

                            // StatsDone - first valid stats message
                            if team.unlock_achievement(AchievementType::StatsDone) {
                                celebrations.push(AchievementType::StatsDone);
                                should_persist = true;
                            }
                        }

                        // Add celebrations and persist if needed
                        for achievement in &celebrations {
                            app_state.add_celebration(team_name.clone(), *achievement);
                        }

                        if should_persist {
                            if let Some(team) = app_state.teams.get(&team_name) {
                                let team_clone = team.clone();
                                drop(app_state);

                                let _ = persistence::persist_state(
                                    &producer,
                                    &scorer_topic,
                                    &team_clone,
                                )
                                .await;
                            }
                        }
                    }
                }
            }
            Err(e) => {
                warn!("Error consuming team_stats: {:?}", e);
            }
        }
    }

    Ok(())
}

/// Monitor a single team's consumer group (runs as independent task)
async fn monitor_single_team(
    team_name: String,
    settings: Settings,
    state: Arc<RwLock<AppState>>,
    producer: Arc<FutureProducer>,
) {
    info!("Starting monitoring task for {}", team_name);

    loop {
        // Update last_data_update at start of cycle to show we're actively monitoring
        {
            let mut app_state = state.write().await;
            app_state.last_data_update = Some(Instant::now());
        }

        // Fetch this team's consumer group status (async-safe, won't block runtime)
        let status = admin::fetch_single_group_status_async(
            &settings.kafka.brokers,
            &settings.kafka.username,
            &settings.kafka.password,
            &settings.kafka.security_protocol,
            &settings.kafka.sasl_mechanism,
            &team_name,
        )
        .await;

        // Fetch lag for this team (async-safe)
        let lag = admin::fetch_consumer_group_lag_async(
            &settings.kafka.brokers,
            &settings.kafka.username,
            &settings.kafka.password,
            &settings.kafka.security_protocol,
            &settings.kafka.sasl_mechanism,
            &team_name,
            &settings.topics.new_users,
        )
        .await;

        // Update shared state (scoped to release lock before sleep)
        let persist_data: Option<(TeamState, String)> = {
            let mut app_state = state.write().await;
            app_state
                .consumer_counts
                .insert(team_name.clone(), status.members);
            app_state
                .group_states
                .insert(team_name.clone(), status.state.clone());

            let mut celebrations = Vec::new();

            if let Some(team) = app_state.teams.get_mut(&team_name) {
                // Update lag if we got it
                if let Some(lag_value) = lag {
                    team.update_lag(lag_value);
                }

                // Check Connected achievement
                if status.state == GroupState::Active
                    && team.unlock_achievement(AchievementType::Connected)
                {
                    celebrations.push(AchievementType::Connected);
                }

                // Check Scaled achievement
                if status.members >= settings.achievements.scaled_members
                    && team.unlock_achievement(AchievementType::Scaled)
                {
                    celebrations.push(AchievementType::Scaled);
                }
            }

            // Add celebrations
            for achievement in &celebrations {
                app_state.add_celebration(team_name.clone(), *achievement);
            }

            // Collect data for persistence outside the lock
            if !celebrations.is_empty() {
                app_state
                    .teams
                    .get(&team_name)
                    .map(|team| (team.clone(), settings.topics.scorer_state.clone()))
            } else {
                None
            }
        }; // Lock released here

        // Persist outside the lock
        if let Some((team_clone, scorer_topic)) = persist_data {
            let _ = persistence::persist_state(&producer, &scorer_topic, &team_clone).await;
        }

        // Sleep before next check
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}
