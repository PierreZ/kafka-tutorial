use crate::config::Settings;
use crate::kafka::{admin, consumer::create_consumer, producer::create_producer};
use crate::state::achievements::AchievementType;
use crate::state::persistence::{persist_state, restore_states};
use crate::state::team::{ConsumerGroupStatus, GroupState, TeamState};
use crate::validation::rules::{
    extract_team_from_payload, validate_action_simple, SimpleValidationResult,
};

use super::widgets::LeaderboardWidget;

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
    Frame, Terminal,
};
use rdkafka::consumer::Consumer;
use rdkafka::message::Message;
use serde::Deserialize;
use std::collections::HashMap;
use std::io;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

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

pub async fn run(settings: Settings) -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Initialize shared state
    let state = Arc::new(RwLock::new(AppState::new(settings.clone())));

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

                            // Step 4: Scaled (2+ consumers)
                            if status.members >= 2
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

                            // PartitionExplorer: 3+ consumers (teaches partition limits)
                            if status.members >= 3
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

                            // LagBuster: Had 100+ lag and now at 0
                            if team.qualifies_for_lag_buster()
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
    let result = run_ui(&mut terminal, Arc::clone(&state)).await;

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
) -> Result<()> {
    let tick_rate = Duration::from_millis(250);

    loop {
        // Draw UI
        {
            let state_guard = state.read().await;
            terminal.draw(|f| draw_ui(f, &state_guard))?;
        }

        // Handle input with timeout
        if event::poll(tick_rate)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                        _ => {}
                    }
                }
            }
        }
    }
}

fn draw_ui(f: &mut Frame, state: &AppState) {
    let size = f.area();

    // Simple layout: full-width table
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0)])
        .split(size);

    // Build consumer groups map
    let consumer_map = LeaderboardWidget::build_consumer_map(&state.consumer_groups);

    // Render leaderboard table
    let sorted_teams = state.get_sorted_teams();
    let leaderboard = LeaderboardWidget::new(&sorted_teams, &consumer_map);
    leaderboard.render(f, chunks[0]);
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

                                // Unlock FirstLoad on first valid action
                                if team_state.unlock_achievement(AchievementType::FirstLoad) {
                                    info!("{} unlocked FirstLoad achievement", team);
                                }

                                // HighThroughput: 100+ valid actions
                                if team_state.action_count >= 100
                                    && team_state
                                        .unlock_achievement(AchievementType::HighThroughput)
                                {
                                    info!("{} unlocked HighThroughput achievement!", team);
                                }

                                // CleanStreak: 50+ messages with 0 errors
                                if team_state.action_count >= 50
                                    && team_state.total_errors() == 0
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
