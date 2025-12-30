use crate::config::Settings;
use crate::kafka::{admin, consumer::create_consumer, producer::create_producer};
use crate::state::achievements::AchievementType;
use crate::state::persistence::{persist_state, restore_states};
use crate::state::team::{ConsumerGroupStatus, GroupState, TeamState};
use crate::validation::rules::{validate_action, Action, User};

use super::widgets::{
    achievements::AchievementNotification, AchievementsWidget, ConsumerGroupsWidget,
    LeaderboardWidget, WatchlistWidget,
};
use super::widgets::watchlist::WatchlistEntry;

use anyhow::Result;
use chrono::Utc;
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
    widgets::{Block, Borders, Paragraph},
    Frame, Terminal,
};
use rdkafka::consumer::Consumer;
use rdkafka::message::Message;
use std::collections::{HashMap, HashSet, VecDeque};
use std::io;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

/// Application state shared between tasks
pub struct AppState {
    pub teams: HashMap<String, TeamState>,
    pub user_cache: HashMap<String, User>,
    pub processed_actions: HashSet<(String, String)>, // (customer, team)
    pub notifications: VecDeque<AchievementNotification>,
    pub consumer_groups: Vec<ConsumerGroupStatus>,
    pub watchlist: Vec<WatchlistEntry>,
    pub settings: Settings,
}

impl AppState {
    pub fn new(settings: Settings) -> Self {
        // Initialize all 15 teams
        let mut teams = HashMap::new();
        for i in 1..=15 {
            let team_name = format!("team-{}", i);
            teams.insert(team_name.clone(), TeamState::new(team_name));
        }

        Self {
            teams,
            user_cache: HashMap::new(),
            processed_actions: HashSet::new(),
            notifications: VecDeque::with_capacity(50),
            consumer_groups: Vec::new(),
            watchlist: Vec::new(),
            settings,
        }
    }

    pub fn get_sorted_teams(&self) -> Vec<TeamState> {
        let mut teams: Vec<_> = self.teams.values().cloned().collect();
        teams.sort_by(|a, b| b.score.cmp(&a.score));
        teams
    }

    /// Add notifications and trim the queue
    pub fn add_notifications(&mut self, team_name: &str, achievements: Vec<AchievementType>) {
        for achievement in achievements {
            self.notifications.push_front(AchievementNotification {
                team_name: team_name.to_string(),
                achievement,
                timestamp: Utc::now(),
            });
        }
        while self.notifications.len() > 50 {
            self.notifications.pop_back();
        }
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

    // Create admin client for consumer group monitoring
    let admin = admin::create_admin_client(
        &settings.kafka.brokers,
        &settings.kafka.username,
        &settings.kafka.password,
    )?;

    // Spawn new_users consumer task
    let state_clone = Arc::clone(&state);
    let settings_clone = settings.clone();
    let new_users_task = tokio::spawn(async move {
        if let Err(e) = consume_new_users(state_clone, settings_clone).await {
            error!("new_users consumer error: {:?}", e);
        }
    });

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
    let watchlist_task = tokio::spawn(async move {
        if let Err(e) = consume_watchlist(state_clone, settings_clone).await {
            error!("watchlist consumer error: {:?}", e);
        }
    });

    // Spawn consumer group monitoring task
    let state_clone = Arc::clone(&state);
    let admin_task = tokio::spawn(async move {
        loop {
            let statuses = admin::fetch_consumer_group_statuses(&admin).await;

            // Update state and check for infrastructure achievements
            {
                let mut state_guard = state_clone.write().await;
                state_guard.consumer_groups = statuses.clone();

                // Collect achievements to add (to avoid borrow conflicts)
                let mut achievements_to_add: Vec<(String, AchievementType)> = Vec::new();

                for status in &statuses {
                    if let Some(team) = state_guard.teams.get_mut(&status.team_name) {
                        if status.state == GroupState::Active {
                            if team.add_infrastructure_achievement(AchievementType::Connected) {
                                achievements_to_add.push((status.team_name.clone(), AchievementType::Connected));
                            }
                            if status.members >= 2 {
                                if team.add_infrastructure_achievement(AchievementType::Scaled) {
                                    achievements_to_add.push((status.team_name.clone(), AchievementType::Scaled));
                                }
                            }
                        }
                    }
                }

                // Now add the notifications (team borrow is released)
                for (team_name, achievement) in achievements_to_add {
                    state_guard.add_notifications(&team_name, vec![achievement]);
                }
            }

            tokio::time::sleep(Duration::from_secs(10)).await;
        }
    });

    // Run UI loop
    let result = run_ui(&mut terminal, Arc::clone(&state)).await;

    // Cleanup
    new_users_task.abort();
    actions_task.abort();
    watchlist_task.abort();
    admin_task.abort();

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

    // Main layout: header + content
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(size);

    // Header
    let title = Paragraph::new("  KAFKA TUTORIAL LEADERBOARD  ")
        .style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, main_chunks[0]);

    // Content layout: left (leaderboard) + right (groups)
    let content_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(main_chunks[1]);

    // Left side: leaderboard + achievements
    let left_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(content_chunks[0]);

    // Leaderboard
    let sorted_teams = state.get_sorted_teams();
    let leaderboard = LeaderboardWidget::new(&sorted_teams);
    leaderboard.render(f, left_chunks[0]);

    // Achievements
    let achievements = AchievementsWidget::new(&state.notifications, 10);
    achievements.render(f, left_chunks[1]);

    // Right side: consumer groups + watchlist
    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(content_chunks[1]);

    // Consumer groups
    let groups = ConsumerGroupsWidget::new(&state.consumer_groups);
    groups.render(f, right_chunks[0]);

    // Watchlist
    let watchlist = WatchlistWidget::new(&state.watchlist);
    watchlist.render(f, right_chunks[1]);
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
                    match serde_json::from_slice::<User>(payload) {
                        Ok(user) => {
                            let mut state_guard = state.write().await;
                            // Use user_name as the key (matches action.customer field)
                            state_guard
                                .user_cache
                                .insert(user.user_name.clone(), user);

                            // Trim cache if too large
                            let max_size = state_guard.settings.scoring.user_cache_max_size;
                            if state_guard.user_cache.len() > max_size {
                                // Simple eviction: remove oldest entries
                                let to_remove = state_guard.user_cache.len() - max_size;
                                let keys: Vec<_> = state_guard
                                    .user_cache
                                    .keys()
                                    .take(to_remove)
                                    .cloned()
                                    .collect();
                                for key in keys {
                                    state_guard.user_cache.remove(&key);
                                }
                            }
                        }
                        Err(e) => {
                            debug!("Failed to parse user: {:?}", e);
                        }
                    }
                }
            }
            Err(e) => {
                warn!("Error consuming new_users: {:?}", e);
            }
        }
    }

    Ok(())
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
                    // Try to parse as Action
                    let action_result = serde_json::from_slice::<Action>(payload);

                    let mut state_guard = state.write().await;

                    match action_result {
                        Ok(action) => {
                            // Validate the action
                            let user = state_guard.user_cache.get(&action.customer);
                            let validation_result = validate_action(
                                &action,
                                user,
                                &state_guard.processed_actions,
                                state_guard.settings.scoring.points_per_correct,
                            );

                            // Process the result for the team and collect data
                            let (new_achievements, team_state_clone) = {
                                if let Some(team) = state_guard.teams.get_mut(&action.team) {
                                    let new_achievements = team.process_result(&validation_result);
                                    let team_state_clone = team.clone();
                                    (new_achievements, Some(team_state_clone))
                                } else {
                                    (Vec::new(), None)
                                }
                            };

                            // Add notifications (team borrow is released)
                            if !new_achievements.is_empty() {
                                state_guard.add_notifications(&action.team, new_achievements);
                            }

                            // Persist state
                            if let Some(team_state) = team_state_clone {
                                let topic = settings.topics.scorer_state.clone();
                                let producer_clone = producer.clone();
                                tokio::spawn(async move {
                                    let _ = persist_state(&producer_clone, &topic, &team_state).await;
                                });
                            }

                            // Mark action as processed (if valid team)
                            if crate::validation::rules::TeamRule::from_team_name(&action.team).is_some() {
                                state_guard
                                    .processed_actions
                                    .insert((action.customer.clone(), action.team.clone()));
                            }
                        }
                        Err(_) => {
                            // Invalid JSON - this is a mistake but we don't know which team
                            // We could try to extract team from partial JSON, but for now skip
                            debug!("Invalid JSON in actions topic");
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

async fn consume_watchlist(state: Arc<RwLock<AppState>>, settings: Settings) -> Result<()> {
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

                            // Update or add watchlist entry (keyed by company)
                            if let Some(existing) = state_guard
                                .watchlist
                                .iter_mut()
                                .find(|e| e.company == entry.company && e.team == entry.team)
                            {
                                existing.flag_count = entry.flag_count;
                            } else {
                                state_guard.watchlist.push(entry);
                            }

                            // Sort by flag count descending
                            state_guard.watchlist.sort_by(|a, b| b.flag_count.cmp(&a.flag_count));

                            // Limit size
                            state_guard.watchlist.truncate(50);
                        }
                        Err(e) => {
                            debug!("Failed to parse watchlist entry: {:?}", e);
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
