use crate::config::Settings;
use crate::kafka::{admin, consumer::create_consumer, producer::create_producer};
use crate::state::achievements::AchievementType;
use crate::state::persistence::{persist_state, restore_states};
use crate::state::team::{ConsumerGroupStatus, GroupState, TeamState};
use crate::validation::rules::{
    extract_team_from_payload, validate_action_simple, SimpleValidationResult,
};

use super::event_buffer::{BufferedEvent, EventBuffer};
use super::widgets::{EventViewWidget, HelpWidget, LeaderboardWidget, TeamDetailWidget};

use kafka_common::WatchlistEntry;

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
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, Borders, TableState, Tabs},
    Frame, Terminal,
};
use rdkafka::consumer::Consumer;
use rdkafka::message::Message;
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

/// Current tab selection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TopicTab {
    #[default]
    Leaderboard,
    NewUsers,
    Actions,
    Watchlist,
}

impl TopicTab {
    pub fn title(&self) -> &'static str {
        match self {
            TopicTab::Leaderboard => "1: Leaderboard",
            TopicTab::NewUsers => "2: new_users",
            TopicTab::Actions => "3: actions",
            TopicTab::Watchlist => "4: watchlist",
        }
    }

    pub fn all() -> [TopicTab; 4] {
        [
            TopicTab::Leaderboard,
            TopicTab::NewUsers,
            TopicTab::Actions,
            TopicTab::Watchlist,
        ]
    }
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
    /// Currently selected tab
    pub current_tab: TopicTab,
    /// Scroll offset for event views (lines from bottom)
    pub event_scroll: usize,
}

impl UiState {
    pub fn new() -> Self {
        Self {
            selected_row: None,
            view_mode: ViewMode::Leaderboard,
            last_data_update: None,
            table_state: TableState::default(),
            current_tab: TopicTab::default(),
            event_scroll: 0,
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

// WatchlistEntry is imported from kafka_common

/// Application state shared between tasks
#[allow(dead_code)]
pub struct AppState {
    pub teams: HashMap<String, TeamState>,
    pub consumer_groups: Vec<ConsumerGroupStatus>,
    pub settings: Settings,
    pub first_blood_awarded: bool, // Track if FirstBlood has been given
    pub champion_awarded: bool,    // Track if Champion has been given
    pub topic_high_watermark: i64, // Track new_users topic high watermark for lag calculation
    /// Event buffers for each topic (150 events each)
    pub new_users_events: EventBuffer,
    pub actions_events: EventBuffer,
    pub watchlist_events: EventBuffer,
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
            new_users_events: EventBuffer::new(150),
            actions_events: EventBuffer::new(150),
            watchlist_events: EventBuffer::new(150),
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
    use crate::config::{
        AchievementSettings, KafkaSettings, ScoringSettings, Settings, TopicSettings,
    };

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

    // Add demo events for the event tabs
    use crate::ui::event_buffer::BufferedEvent;
    use kafka_common::{Action, User, WatchlistEntry};

    // Demo new_users events
    let demo_users = vec![
        User {
            email: "alice@techcorp.io".to_string(),
            credit_card_number: "4532-****-****-1234".to_string(),
            company_name: "TechCorp".to_string(),
            company_slogan: "Innovation First".to_string(),
            industry: "Technology".to_string(),
            user_name: "alice_tech".to_string(),
            avatar: "https://example.com/avatar1.png".to_string(),
            name: "Alice Johnson".to_string(),
            profession: "Software Engineer".to_string(),
            field: "Backend Development".to_string(),
            premium: true,
            credit: 1500,
            time_zone: "Europe/Paris".to_string(),
            user_agent: "Mozilla/5.0".to_string(),
            pack: "gold".to_string(),
        },
        User {
            email: "bob@startup.co".to_string(),
            credit_card_number: "5412-****-****-5678".to_string(),
            company_name: "StartupCo".to_string(),
            company_slogan: "Move Fast".to_string(),
            industry: "Finance".to_string(),
            user_name: "bob_startup".to_string(),
            avatar: "https://example.com/avatar2.png".to_string(),
            name: "Bob Smith".to_string(),
            profession: "Data Analyst".to_string(),
            field: "Analytics".to_string(),
            premium: false,
            credit: 250,
            time_zone: "America/New_York".to_string(),
            user_agent: "Chrome/120.0".to_string(),
            pack: "basic".to_string(),
        },
        User {
            email: "charlie@megacorp.com".to_string(),
            credit_card_number: "3782-****-****-9012".to_string(),
            company_name: "MegaCorp".to_string(),
            company_slogan: "Scale Everything".to_string(),
            industry: "Healthcare".to_string(),
            user_name: "charlie_mega".to_string(),
            avatar: "https://example.com/avatar3.png".to_string(),
            name: "Charlie Brown".to_string(),
            profession: "Product Manager".to_string(),
            field: "Product".to_string(),
            premium: true,
            credit: 3000,
            time_zone: "Asia/Tokyo".to_string(),
            user_agent: "Safari/17.0".to_string(),
            pack: "enterprise".to_string(),
        },
    ];

    for user in demo_users {
        let json = serde_json::to_string(&user).unwrap_or_default();
        app_state
            .new_users_events
            .push(BufferedEvent::new(json, None));
    }

    // Demo actions events
    let demo_actions = vec![
        Action {
            customer: Some("alice@techcorp.io".to_string()),
            action_type: Some("CONTACT".to_string()),
            reason: Some("VIP customer".to_string()),
            team: "team-1".to_string(),
        },
        Action {
            customer: Some("bob@startup.co".to_string()),
            action_type: Some("FLAG".to_string()),
            reason: Some("Suspicious activity".to_string()),
            team: "team-2".to_string(),
        },
        Action {
            customer: Some("charlie@megacorp.com".to_string()),
            action_type: Some("CONTACT".to_string()),
            reason: Some("High value".to_string()),
            team: "team-1".to_string(),
        },
        Action {
            customer: Some("dave@example.org".to_string()),
            action_type: Some("FLAG".to_string()),
            reason: Some("Premium expiring".to_string()),
            team: "team-3".to_string(),
        },
        Action {
            customer: Some("eve@company.net".to_string()),
            action_type: Some("CONTACT".to_string()),
            reason: Some("New signup".to_string()),
            team: "team-5".to_string(),
        },
    ];

    for action in demo_actions {
        let team = action.team.clone();
        let json = serde_json::to_string(&action).unwrap_or_default();
        app_state
            .actions_events
            .push(BufferedEvent::new(json, Some(team)));
    }

    // Demo watchlist events
    let demo_watchlist = vec![
        WatchlistEntry {
            team: "team-1".to_string(),
            company: "TechCorp".to_string(),
            flag_count: 3,
        },
        WatchlistEntry {
            team: "team-2".to_string(),
            company: "StartupCo".to_string(),
            flag_count: 7,
        },
        WatchlistEntry {
            team: "team-5".to_string(),
            company: "MegaCorp".to_string(),
            flag_count: 2,
        },
    ];

    for entry in demo_watchlist {
        let team = entry.team.clone();
        let json = serde_json::to_string(&entry).unwrap_or_default();
        app_state
            .watchlist_events
            .push(BufferedEvent::new(json, Some(team)));
    }

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

    // Spawn new_users consumer task (for event display only)
    let state_clone = Arc::clone(&state);
    let settings_clone = settings.clone();
    let new_users_task = tokio::spawn(async move {
        if let Err(e) = consume_new_users(state_clone, settings_clone).await {
            error!("new_users consumer error: {:?}", e);
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
                            if status.members
                                >= settings_clone.achievements.partition_explorer_members
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
                            if team.qualifies_for_lag_buster(
                                settings_clone.achievements.lag_buster_threshold,
                            ) && team.unlock_achievement(AchievementType::LagBuster)
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
    new_users_task.abort();
    admin_task.abort();

    // Wait briefly for tasks to complete their abort handling
    let shutdown_timeout = Duration::from_millis(100);
    let _ = tokio::time::timeout(shutdown_timeout, async {
        let _ = tokio::join!(actions_task, watchlist_task, new_users_task, admin_task);
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
                    let (team_count, event_buffer_len) = {
                        let state_guard = state.read().await;
                        let buffer_len = match ui_guard.current_tab {
                            TopicTab::NewUsers => state_guard.new_users_events.len(),
                            TopicTab::Actions => state_guard.actions_events.len(),
                            TopicTab::Watchlist => state_guard.watchlist_events.len(),
                            TopicTab::Leaderboard => 0,
                        };
                        (state_guard.teams.len(), buffer_len)
                    };

                    // Handle overlays first (Help, TeamDetail)
                    match ui_guard.view_mode {
                        ViewMode::TeamDetail => {
                            match key.code {
                                KeyCode::Esc | KeyCode::Backspace | KeyCode::Char('q') => {
                                    ui_guard.view_mode = ViewMode::Leaderboard;
                                }
                                _ => {}
                            }
                            continue;
                        }
                        ViewMode::Help => {
                            match key.code {
                                KeyCode::Esc
                                | KeyCode::Char('h')
                                | KeyCode::Char('?')
                                | KeyCode::Char('q') => {
                                    ui_guard.view_mode = ViewMode::Leaderboard;
                                }
                                _ => {}
                            }
                            continue;
                        }
                        ViewMode::Leaderboard => {}
                    }

                    // Global quit
                    if matches!(key.code, KeyCode::Char('q') | KeyCode::Esc) {
                        return Ok(());
                    }

                    // Tab switching (1-4)
                    match key.code {
                        KeyCode::Char('1') => {
                            ui_guard.current_tab = TopicTab::Leaderboard;
                            ui_guard.event_scroll = 0;
                        }
                        KeyCode::Char('2') => {
                            ui_guard.current_tab = TopicTab::NewUsers;
                            ui_guard.event_scroll = 0;
                        }
                        KeyCode::Char('3') => {
                            ui_guard.current_tab = TopicTab::Actions;
                            ui_guard.event_scroll = 0;
                        }
                        KeyCode::Char('4') => {
                            ui_guard.current_tab = TopicTab::Watchlist;
                            ui_guard.event_scroll = 0;
                        }
                        _ => {}
                    }

                    // Tab-specific navigation
                    match ui_guard.current_tab {
                        TopicTab::Leaderboard => match key.code {
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
                        TopicTab::NewUsers | TopicTab::Actions | TopicTab::Watchlist => {
                            match key.code {
                                KeyCode::Up | KeyCode::Char('k') => {
                                    // Scroll up = show older events
                                    ui_guard.event_scroll = ui_guard
                                        .event_scroll
                                        .saturating_add(1)
                                        .min(event_buffer_len.saturating_sub(1));
                                }
                                KeyCode::Down | KeyCode::Char('j') => {
                                    // Scroll down = show newer events
                                    ui_guard.event_scroll = ui_guard.event_scroll.saturating_sub(1);
                                }
                                KeyCode::Home => {
                                    // Jump to newest (scroll = 0)
                                    ui_guard.event_scroll = 0;
                                }
                                KeyCode::End => {
                                    // Jump to oldest
                                    ui_guard.event_scroll = event_buffer_len.saturating_sub(1);
                                }
                                KeyCode::Char('h') | KeyCode::Char('?') => {
                                    ui_guard.view_mode = ViewMode::Help;
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
        }
    }
}

fn draw_ui(f: &mut Frame, state: &AppState, ui_state: &mut UiState) {
    let size = f.area();

    // Layout: tabs at top, content below
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(size);

    // Render tab bar
    render_tabs(f, chunks[0], ui_state.current_tab);

    // Render content based on current tab
    match ui_state.current_tab {
        TopicTab::Leaderboard => {
            // Build consumer groups map
            let consumer_map = LeaderboardWidget::build_consumer_map(&state.consumer_groups);

            // Render leaderboard table with selection
            let sorted_teams = state.get_sorted_teams();
            let leaderboard =
                LeaderboardWidget::new(&sorted_teams, &consumer_map, ui_state.last_data_update);
            leaderboard.render_stateful(f, chunks[1], &mut ui_state.table_state);

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
        TopicTab::NewUsers => {
            EventViewWidget::new(&state.new_users_events, "new_users", ui_state.event_scroll)
                .render(f, chunks[1]);
        }
        TopicTab::Actions => {
            EventViewWidget::new(&state.actions_events, "actions", ui_state.event_scroll)
                .render(f, chunks[1]);
        }
        TopicTab::Watchlist => {
            EventViewWidget::new(&state.watchlist_events, "watchlist", ui_state.event_scroll)
                .render(f, chunks[1]);
        }
    }
}

fn render_tabs(f: &mut Frame, area: ratatui::layout::Rect, current: TopicTab) {
    let titles: Vec<Line> = TopicTab::all()
        .iter()
        .map(|t| Line::from(t.title()))
        .collect();

    let selected = match current {
        TopicTab::Leaderboard => 0,
        TopicTab::NewUsers => 1,
        TopicTab::Actions => 2,
        TopicTab::Watchlist => 3,
    };

    let tabs = Tabs::new(titles)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Kafka Events "),
        )
        .select(selected)
        .style(Style::default().fg(Color::White))
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .divider("|");

    f.render_widget(tabs, area);
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

                    // Buffer event for display
                    let json_str = String::from_utf8_lossy(payload).to_string();
                    let compact = serde_json::from_str::<serde_json::Value>(&json_str)
                        .map(|v| serde_json::to_string(&v).unwrap_or_else(|_| json_str.clone()))
                        .unwrap_or_else(|_| json_str);
                    let team_for_event = match &validation {
                        SimpleValidationResult::Valid { team } => Some(team.clone()),
                        SimpleValidationResult::MissingFields { team } => Some(team.clone()),
                        SimpleValidationResult::InvalidJson => extract_team_from_payload(payload),
                    };
                    state_guard
                        .actions_events
                        .push(BufferedEvent::new(compact, team_for_event));

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
                                if team_state.action_count
                                    >= settings.achievements.high_throughput_threshold
                                    && team_state
                                        .unlock_achievement(AchievementType::HighThroughput)
                                {
                                    info!("{} unlocked HighThroughput achievement!", team);
                                }

                                // CleanStreak: configurable consecutive valid messages (default 50)
                                if team_state.clean_streak_count
                                    >= settings.achievements.clean_streak_threshold
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

                            // Buffer event for display
                            let json_str = String::from_utf8_lossy(payload).to_string();
                            let compact = serde_json::from_str::<serde_json::Value>(&json_str)
                                .map(|v| {
                                    serde_json::to_string(&v).unwrap_or_else(|_| json_str.clone())
                                })
                                .unwrap_or_else(|_| json_str);
                            state_guard
                                .watchlist_events
                                .push(BufferedEvent::new(compact, Some(entry.team.clone())));

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

async fn consume_new_users(state: Arc<RwLock<AppState>>, settings: Settings) -> Result<()> {
    let consumer = create_consumer(
        &settings.kafka.brokers,
        &settings.kafka.username,
        &settings.kafka.password,
        &format!("{}-new-users", settings.kafka.consumer_group),
    )?;

    consumer.subscribe(&[&settings.topics.new_users])?;
    info!("Subscribed to topic: {}", settings.topics.new_users);

    let mut stream = consumer.stream();

    while let Some(result) = stream.next().await {
        match result {
            Ok(message) => {
                if let Some(payload) = message.payload() {
                    // Convert to compact JSON string
                    let json_str = String::from_utf8_lossy(payload).to_string();
                    let compact = serde_json::from_str::<serde_json::Value>(&json_str)
                        .map(|v| serde_json::to_string(&v).unwrap_or_else(|_| json_str.clone()))
                        .unwrap_or_else(|_| json_str);

                    // No team for new_users (source data from producer)
                    let event = BufferedEvent::new(compact, None);

                    let mut state_guard = state.write().await;
                    state_guard.new_users_events.push(event);
                }
            }
            Err(e) => {
                warn!("Error consuming new_users: {:?}", e);
            }
        }
    }

    Ok(())
}
