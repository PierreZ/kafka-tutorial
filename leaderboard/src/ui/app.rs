use crate::config::Settings;
use crate::kafka::{admin, consumer::create_consumer, producer::create_producer};
use crate::state::achievements::AchievementType;
use crate::state::persistence::{persist_state, restore_states};
use crate::state::team::{ConsumerGroupStatus, GroupState, TeamState};
use crate::validation::rules::{
    extract_team_from_payload, validate_action_simple, SimpleValidationResult,
};

use super::widgets::{HelpWidget, LeaderboardWidget, TeamDetailWidget};

use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use futures::StreamExt;
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    widgets::TableState,
    Frame, Terminal,
};
use rdkafka::consumer::Consumer;
use rdkafka::message::Message;
use serde::Deserialize;
use std::collections::HashMap;
use std::io;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

/// Current view mode for the UI
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewMode {
    Leaderboard,
    TeamDetail,
    Help,
}

/// UI-specific state for navigation and display modes
pub struct UiState {
    /// Currently selected row index
    pub selected_row: Option<usize>,
    /// Current view mode
    pub view_mode: ViewMode,
    /// Last time data was updated from Kafka
    pub last_data_update: Option<Instant>,
    /// Table state for StatefulWidget rendering
    pub table_state: TableState,
}

impl UiState {
    pub fn new() -> Self {
        Self {
            selected_row: None,
            view_mode: ViewMode::Leaderboard,
            last_data_update: None,
            table_state: TableState::default(),
        }
    }

    pub fn move_selection_up(&mut self, max: usize) {
        if max == 0 {
            return;
        }
        let new_index = match self.selected_row {
            Some(i) if i > 0 => i - 1,
            Some(_) => max - 1, // Wrap to end
            None => 0,          // Start at top
        };
        self.selected_row = Some(new_index);
        self.table_state.select(Some(new_index));
    }

    pub fn move_selection_down(&mut self, max: usize) {
        if max == 0 {
            return;
        }
        let new_index = match self.selected_row {
            Some(i) if i < max - 1 => i + 1,
            Some(_) => 0, // Wrap to start
            None => 0,    // Start at top
        };
        self.selected_row = Some(new_index);
        self.table_state.select(Some(new_index));
    }

    pub fn select_first(&mut self) {
        self.selected_row = Some(0);
        self.table_state.select(Some(0));
    }

    pub fn select_last(&mut self, max: usize) {
        if max > 0 {
            self.selected_row = Some(max - 1);
            self.table_state.select(Some(max - 1));
        }
    }
}

/// Watchlist entry from students
#[derive(Debug, Clone, Deserialize)]
pub struct WatchlistEntry {
    pub team: String,
    #[allow(dead_code)]
    pub company: String,
    #[allow(dead_code)]
    pub flag_count: u32,
}

/// Application state shared between tasks
#[allow(dead_code)]
pub struct AppState {
    pub teams: HashMap<String, TeamState>,
    pub consumer_groups: Vec<ConsumerGroupStatus>,
    pub settings: Settings,
    pub first_blood_awarded: bool, // Track if FirstBlood has been given
    pub champion_awarded: bool,    // Track if Champion has been given
    pub topic_high_watermark: i64, // Track new_users topic high watermark for lag calculation
}

impl AppState {
    pub fn new(settings: Settings) -> Self {
        // Initialize all teams
        let mut teams = HashMap::new();
        for i in 1..=crate::NUM_TEAMS {
            let team_name = format!("team-{}", i);
            teams.insert(team_name.clone(), TeamState::new(team_name));
        }

        Self {
            teams,
            consumer_groups: Vec::new(),
            settings,
            first_blood_awarded: false,
            champion_awarded: false,
            topic_high_watermark: 0,
        }
    }

    pub fn get_sorted_teams(&self) -> Vec<TeamState> {
        let mut teams: Vec<_> = self.teams.values().cloned().collect();
        // Sort by step count descending, then by action count, then by name
        teams.sort_by(|a, b| {
            b.step_count()
                .cmp(&a.step_count())
                .then_with(|| b.action_count.cmp(&a.action_count))
                .then_with(|| a.team_name.cmp(&b.team_name))
        });
        teams
    }
}

/// Run in demo mode without Kafka (for UI testing)
pub async fn run_demo() -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create demo state with sample data
    let state = Arc::new(RwLock::new(create_demo_state()));
    let ui_state = Arc::new(RwLock::new(UiState::new()));

    // Set initial last update time
    {
        let mut ui_guard = ui_state.write().await;
        ui_guard.last_data_update = Some(Instant::now());
    }

    // Run UI loop only (no Kafka tasks)
    let result = run_ui(&mut terminal, Arc::clone(&state), Arc::clone(&ui_state)).await;

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

fn create_demo_state() -> AppState {
    use crate::config::{AchievementSettings, KafkaSettings, ScoringSettings, Settings, TopicSettings};

    let settings = Settings {
        kafka: KafkaSettings {
            brokers: "demo".to_string(),
            username: "demo".to_string(),
            password: "demo".to_string(),
            consumer_group: "demo".to_string(),
        },
        topics: TopicSettings {
            new_users: "new_users".to_string(),
            actions: "actions".to_string(),
            watchlist: "watchlist".to_string(),
            scorer_state: "scorer_state".to_string(),
        },
        scoring: ScoringSettings::default(),
        achievements: AchievementSettings::default(),
    };

    let mut app_state = AppState::new(settings);

    // Add demo data to some teams
    if let Some(team) = app_state.teams.get_mut("team-1") {
        team.unlock_achievement(AchievementType::Connected);
        team.unlock_achievement(AchievementType::FirstLoad);
        team.unlock_achievement(AchievementType::Scaled);
        team.unlock_achievement(AchievementType::WatchlistDone);
        team.unlock_achievement(AchievementType::HighThroughput);
        team.unlock_achievement(AchievementType::CleanStreak);
        team.unlock_achievement(AchievementType::FirstBlood);
        team.action_count = 150;
        team.watchlist_count = 5;
    }

    if let Some(team) = app_state.teams.get_mut("team-2") {
        team.unlock_achievement(AchievementType::Connected);
        team.unlock_achievement(AchievementType::FirstLoad);
        team.unlock_achievement(AchievementType::Scaled);
        team.unlock_achievement(AchievementType::PartitionExplorer);
        team.action_count = 85;
        team.record_error(AchievementType::ParseError);
        team.record_error(AchievementType::ParseError);
    }

    if let Some(team) = app_state.teams.get_mut("team-3") {
        team.unlock_achievement(AchievementType::Connected);
        team.unlock_achievement(AchievementType::FirstLoad);
        team.action_count = 42;
        team.record_error(AchievementType::MissingFields);
    }

    if let Some(team) = app_state.teams.get_mut("team-4") {
        team.unlock_achievement(AchievementType::Connected);
        team.action_count = 10;
    }

    if let Some(team) = app_state.teams.get_mut("team-5") {
        team.unlock_achievement(AchievementType::Connected);
        team.unlock_achievement(AchievementType::FirstLoad);
        team.unlock_achievement(AchievementType::Scaled);
        team.unlock_achievement(AchievementType::WatchlistDone);
        team.unlock_achievement(AchievementType::LagBuster);
        team.action_count = 75;
        team.watchlist_count = 3;
        team.max_lag_seen = 150;
    }

    // Add consumer group statuses for demo
    app_state.consumer_groups = vec![
        ConsumerGroupStatus {
            team_name: "team-1".to_string(),
            state: GroupState::Active,
            members: 2,
            lag: 0,
        },
        ConsumerGroupStatus {
            team_name: "team-2".to_string(),
            state: GroupState::Active,
            members: 3,
            lag: 5,
        },
        ConsumerGroupStatus {
            team_name: "team-3".to_string(),
            state: GroupState::Active,
            members: 1,
            lag: 12,
        },
        ConsumerGroupStatus {
            team_name: "team-4".to_string(),
            state: GroupState::Active,
            members: 1,
            lag: 50,
        },
        ConsumerGroupStatus {
            team_name: "team-5".to_string(),
            state: GroupState::Active,
            members: 2,
            lag: 0,
        },
    ];

    app_state
}

pub async fn run(settings: Settings) -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Initialize shared state
    let state = Arc::new(RwLock::new(AppState::new(settings.clone())));
    let ui_state = Arc::new(RwLock::new(UiState::new()));

    // Try to restore state from Kafka
    info!("Attempting to restore state from Kafka...");
    match restore_states(
        &settings.kafka.brokers,
        &settings.kafka.username,
        &settings.kafka.password,
        &settings.topics.scorer_state,
    )
    .await
    {
        Ok(restored) => {
            let mut state_guard = state.write().await;
            for (team_name, team_state) in restored {
                // Check if this team had FirstBlood or Champion before restoring
                if team_state.has_achievement(AchievementType::FirstBlood) {
                    state_guard.first_blood_awarded = true;
                }
                if team_state.has_achievement(AchievementType::Champion) {
                    state_guard.champion_awarded = true;
                }
                state_guard.teams.insert(team_name, team_state);
            }
            info!("State restored successfully");
        }
        Err(e) => {
            warn!("Could not restore state: {:?}", e);
        }
    }

    // Create producer for state persistence
    let producer = create_producer(
        &settings.kafka.brokers,
        &settings.kafka.username,
        &settings.kafka.password,
    )?;

    // Spawn actions consumer task
    let state_clone = Arc::clone(&state);
    let settings_clone = settings.clone();
    let producer_clone = producer.clone();
    let actions_task = tokio::spawn(async move {
        if let Err(e) = consume_actions(state_clone, settings_clone, producer_clone).await {
            error!("actions consumer error: {:?}", e);
        }
    });

    // Spawn watchlist consumer task
    let state_clone = Arc::clone(&state);
    let settings_clone = settings.clone();
    let producer_clone = producer.clone();
    let watchlist_task = tokio::spawn(async move {
        if let Err(e) = consume_watchlist(state_clone, settings_clone, producer_clone).await {
            error!("watchlist consumer error: {:?}", e);
        }
    });

    // Spawn consumer group monitoring task
    let state_clone = Arc::clone(&state);
    let producer_clone = producer.clone();
    let settings_clone = settings.clone();
    let admin_task = tokio::spawn(async move {
        loop {
            let statuses = admin::fetch_consumer_group_statuses(
                &settings_clone.kafka.brokers,
                &settings_clone.kafka.username,
                &settings_clone.kafka.password,
            )
            .await;

            // Try to fetch high watermark for lag calculation (do this outside the lock)
            let watermark = admin::fetch_topic_high_watermark(
                &settings_clone.kafka.brokers,
                &settings_clone.kafka.username,
                &settings_clone.kafka.password,
                &settings_clone.topics.new_users,
            )
            .ok();

            // Update state and check for infrastructure achievements
            {
                let mut state_guard = state_clone.write().await;
                state_guard.consumer_groups = statuses.clone();

                // Update high watermark if we got a new value
                if let Some(wm) = watermark {
                    state_guard.topic_high_watermark = wm;
                }
                let high_watermark = state_guard.topic_high_watermark;

                // Cache global flags before the loop
                let mut first_blood_awarded = state_guard.first_blood_awarded;
                let mut champion_awarded = state_guard.champion_awarded;

                // Check for Connected and Scaled achievements
                for status in &statuses {
                    if let Some(team) = state_guard.teams.get_mut(&status.team_name) {
                        if status.state == GroupState::Active {
                            // Step 1: Connected
                            if team.unlock_achievement(AchievementType::Connected) {
                                info!("{} unlocked Connected achievement", status.team_name);

                                // FirstBlood: First team to get Connected
                                if !first_blood_awarded {
                                    first_blood_awarded = true;
                                    team.unlock_achievement(AchievementType::FirstBlood);
                                    info!("{} unlocked FirstBlood achievement!", status.team_name);
                                }

                                // Persist state
                                let team_clone = team.clone();
                                let topic = settings_clone.topics.scorer_state.clone();
                                let producer = producer_clone.clone();
                                tokio::spawn(async move {
                                    let _ = persist_state(&producer, &topic, &team_clone).await;
                                });
                            }

                            // Step 4: Scaled (configurable consumers, default 2)
                            if status.members >= settings_clone.achievements.scaled_members
                                && team.unlock_achievement(AchievementType::Scaled)
                            {
                                info!("{} unlocked Scaled achievement", status.team_name);
                                let team_clone = team.clone();
                                let topic = settings_clone.topics.scorer_state.clone();
                                let producer = producer_clone.clone();
                                tokio::spawn(async move {
                                    let _ = persist_state(&producer, &topic, &team_clone).await;
                                });
                            }

                            // PartitionExplorer: configurable consumers (default 3, teaches partition limits)
                            if status.members >= settings_clone.achievements.partition_explorer_members
                                && team.unlock_achievement(AchievementType::PartitionExplorer)
                            {
                                info!(
                                    "{} unlocked PartitionExplorer achievement!",
                                    status.team_name
                                );
                                let team_clone = team.clone();
                                let topic = settings_clone.topics.scorer_state.clone();
                                let producer = producer_clone.clone();
                                tokio::spawn(async move {
                                    let _ = persist_state(&producer, &topic, &team_clone).await;
                                });
                            }

                            // Update lag tracking for LagBuster
                            let estimated_lag =
                                admin::estimate_team_lag(high_watermark, team.messages_consumed);
                            team.update_lag(estimated_lag);

                            // LagBuster: Had configurable lag (default 100) and now caught up
                            if team.qualifies_for_lag_buster(settings_clone.achievements.lag_buster_threshold)
                                && team.unlock_achievement(AchievementType::LagBuster)
                            {
                                info!("{} unlocked LagBuster achievement!", status.team_name);
                                let team_clone = team.clone();
                                let topic = settings_clone.topics.scorer_state.clone();
                                let producer = producer_clone.clone();
                                tokio::spawn(async move {
                                    let _ = persist_state(&producer, &topic, &team_clone).await;
                                });
                            }

                            // Champion: First team to have all requirements
                            if !champion_awarded
                                && team.has_all_champion_requirements()
                                && team.unlock_achievement(AchievementType::Champion)
                            {
                                champion_awarded = true;
                                info!("{} unlocked Champion achievement! 🏆", status.team_name);
                                let team_clone = team.clone();
                                let topic = settings_clone.topics.scorer_state.clone();
                                let producer = producer_clone.clone();
                                tokio::spawn(async move {
                                    let _ = persist_state(&producer, &topic, &team_clone).await;
                                });
                            }
                        }
                    }
                }

                // Update global flags after the loop
                state_guard.first_blood_awarded = first_blood_awarded;
                state_guard.champion_awarded = champion_awarded;
            }

            tokio::time::sleep(Duration::from_secs(10)).await;
        }
    });

    // Run UI loop
    let result = run_ui(&mut terminal, Arc::clone(&state), Arc::clone(&ui_state)).await;

    // Graceful shutdown: abort tasks and wait briefly for cleanup
    info!("Shutting down background tasks...");
    actions_task.abort();
    watchlist_task.abort();
    admin_task.abort();

    // Wait briefly for tasks to complete their abort handling
    let shutdown_timeout = Duration::from_millis(100);
    let _ = tokio::time::timeout(shutdown_timeout, async {
        let _ = tokio::join!(actions_task, watchlist_task, admin_task);
    })
    .await;
    debug!("Background tasks shut down");

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

async fn run_ui(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    state: Arc<RwLock<AppState>>,
    ui_state: Arc<RwLock<UiState>>,
) -> Result<()> {
    let tick_rate = Duration::from_millis(250);

    loop {
        // Draw UI
        {
            let state_guard = state.read().await;
            let mut ui_guard = ui_state.write().await;
            terminal.draw(|f| draw_ui(f, &state_guard, &mut ui_guard))?;
        }

        // Handle input with timeout
        if event::poll(tick_rate)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    let mut ui_guard = ui_state.write().await;
                    let team_count = {
                        let state_guard = state.read().await;
                        state_guard.teams.len()
                    };

                    match ui_guard.view_mode {
                        ViewMode::Leaderboard => match key.code {
                            KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                            KeyCode::Up | KeyCode::Char('k') => {
                                ui_guard.move_selection_up(team_count);
                            }
                            KeyCode::Down | KeyCode::Char('j') => {
                                ui_guard.move_selection_down(team_count);
                            }
                            KeyCode::Enter => {
                                if ui_guard.selected_row.is_some() {
                                    ui_guard.view_mode = ViewMode::TeamDetail;
                                }
                            }
                            KeyCode::Char('h') | KeyCode::Char('?') => {
                                ui_guard.view_mode = ViewMode::Help;
                            }
                            KeyCode::Home => {
                                ui_guard.select_first();
                            }
                            KeyCode::End => {
                                ui_guard.select_last(team_count);
                            }
                            _ => {}
                        },
                        ViewMode::TeamDetail => match key.code {
                            KeyCode::Esc | KeyCode::Backspace | KeyCode::Char('q') => {
                                ui_guard.view_mode = ViewMode::Leaderboard;
                            }
                            _ => {}
                        },
                        ViewMode::Help => match key.code {
                            KeyCode::Esc
                            | KeyCode::Char('h')
                            | KeyCode::Char('?')
                            | KeyCode::Char('q') => {
                                ui_guard.view_mode = ViewMode::Leaderboard;
                            }
                            _ => {}
                        },
                    }
                }
            }
        }
    }
}

fn draw_ui(f: &mut Frame, state: &AppState, ui_state: &mut UiState) {
    let size = f.area();

    // Simple layout: full-width table
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0)])
        .split(size);

    // Build consumer groups map
    let consumer_map = LeaderboardWidget::build_consumer_map(&state.consumer_groups);

    // Render leaderboard table with selection
    let sorted_teams = state.get_sorted_teams();
    let leaderboard =
        LeaderboardWidget::new(&sorted_teams, &consumer_map, ui_state.last_data_update);
    leaderboard.render_stateful(f, chunks[0], &mut ui_state.table_state);

    // Render overlays based on view mode
    match ui_state.view_mode {
        ViewMode::Help => {
            HelpWidget::render(f, size);
        }
        ViewMode::TeamDetail => {
            if let Some(selected) = ui_state.selected_row {
                if let Some(team) = sorted_teams.get(selected) {
                    let consumer_status = state
                        .consumer_groups
                        .iter()
                        .find(|s| s.team_name == team.team_name);
                    TeamDetailWidget::new(team, consumer_status).render(f, size);
                }
            }
        }
        ViewMode::Leaderboard => {}
    }
}

async fn consume_actions(
    state: Arc<RwLock<AppState>>,
    settings: Settings,
    producer: rdkafka::producer::FutureProducer,
) -> Result<()> {
    let consumer = create_consumer(
        &settings.kafka.brokers,
        &settings.kafka.username,
        &settings.kafka.password,
        &format!("{}-actions", settings.kafka.consumer_group),
    )?;

    consumer.subscribe(&[&settings.topics.actions])?;
    info!("Subscribed to topic: {}", settings.topics.actions);

    let mut stream = consumer.stream();

    while let Some(result) = stream.next().await {
        match result {
            Ok(message) => {
                if let Some(payload) = message.payload() {
                    let validation = validate_action_simple(payload);

                    let mut state_guard = state.write().await;

                    // Track messages_consumed for lag calculation (all messages, not just valid)
                    if let Some(team) = match &validation {
                        SimpleValidationResult::Valid { team } => Some(team.clone()),
                        SimpleValidationResult::MissingFields { team } => Some(team.clone()),
                        SimpleValidationResult::InvalidJson => extract_team_from_payload(payload),
                    } {
                        if let Some(team_state) = state_guard.teams.get_mut(&team) {
                            team_state.messages_consumed += 1;
                        }
                    }

                    match validation {
                        SimpleValidationResult::Valid { team } => {
                            if let Some(team_state) = state_guard.teams.get_mut(&team) {
                                team_state.action_count += 1;
                                team_state.increment_clean_streak();

                                // Unlock FirstLoad on first valid action
                                if team_state.unlock_achievement(AchievementType::FirstLoad) {
                                    info!("{} unlocked FirstLoad achievement", team);
                                }

                                // HighThroughput: configurable valid actions (default 100)
                                if team_state.action_count >= settings.achievements.high_throughput_threshold
                                    && team_state
                                        .unlock_achievement(AchievementType::HighThroughput)
                                {
                                    info!("{} unlocked HighThroughput achievement!", team);
                                }

                                // CleanStreak: configurable consecutive valid messages (default 50)
                                if team_state.clean_streak_count >= settings.achievements.clean_streak_threshold
                                    && team_state.unlock_achievement(AchievementType::CleanStreak)
                                {
                                    info!("{} unlocked CleanStreak achievement!", team);
                                }

                                // Persist state
                                let team_clone = team_state.clone();
                                let topic = settings.topics.scorer_state.clone();
                                let producer = producer.clone();
                                tokio::spawn(async move {
                                    let _ = persist_state(&producer, &topic, &team_clone).await;
                                });
                            }
                        }
                        SimpleValidationResult::InvalidJson => {
                            // Try to extract team for error tracking
                            if let Some(team) = extract_team_from_payload(payload) {
                                if let Some(team_state) = state_guard.teams.get_mut(&team) {
                                    team_state.record_error(AchievementType::ParseError);
                                }
                            }
                            debug!("Invalid JSON in actions topic");
                        }
                        SimpleValidationResult::MissingFields { team } => {
                            if let Some(team_state) = state_guard.teams.get_mut(&team) {
                                team_state.record_error(AchievementType::MissingFields);
                            }
                        }
                    }
                } else {
                    debug!("Received message with empty payload in actions topic");
                }
            }
            Err(e) => {
                warn!("Error consuming actions: {:?}", e);
            }
        }
    }

    Ok(())
}

async fn consume_watchlist(
    state: Arc<RwLock<AppState>>,
    settings: Settings,
    producer: rdkafka::producer::FutureProducer,
) -> Result<()> {
    let consumer = create_consumer(
        &settings.kafka.brokers,
        &settings.kafka.username,
        &settings.kafka.password,
        &format!("{}-watchlist", settings.kafka.consumer_group),
    )?;

    consumer.subscribe(&[&settings.topics.watchlist])?;
    info!("Subscribed to topic: {}", settings.topics.watchlist);

    let mut stream = consumer.stream();

    while let Some(result) = stream.next().await {
        match result {
            Ok(message) => {
                if let Some(payload) = message.payload() {
                    match serde_json::from_slice::<WatchlistEntry>(payload) {
                        Ok(entry) => {
                            let mut state_guard = state.write().await;

                            if let Some(team_state) = state_guard.teams.get_mut(&entry.team) {
                                team_state.watchlist_count += 1;
                                // Unlock WatchlistDone on first watchlist message
                                if team_state.unlock_achievement(AchievementType::WatchlistDone) {
                                    info!("{} unlocked WatchlistDone achievement", entry.team);
                                    // Persist state
                                    let team_clone = team_state.clone();
                                    let topic = settings.topics.scorer_state.clone();
                                    let producer = producer.clone();
                                    tokio::spawn(async move {
                                        let _ = persist_state(&producer, &topic, &team_clone).await;
                                    });
                                }
                            }
                        }
                        Err(e) => {
                            debug!("Failed to parse watchlist entry: {:?}", e);
                        }
                    }
                } else {
                    debug!("Received message with empty payload in watchlist topic");
                }
            }
            Err(e) => {
                warn!("Error consuming watchlist: {:?}", e);
            }
        }
    }

    Ok(())
}
