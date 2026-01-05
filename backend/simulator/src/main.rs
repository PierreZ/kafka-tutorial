mod config;
mod filters;
mod team;
mod watchlist;

use crate::config::Settings;
use crate::filters::get_filter;
use crate::team::TeamSimulator;
use crate::watchlist::WatchlistState;
use anyhow::Result;
use clap::Parser;
use rdkafka::consumer::StreamConsumer;
use rdkafka::producer::FutureProducer;
use std::fs::File;
use std::sync::Arc;
use tokio::task::JoinSet;
use tracing::{error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

const NUM_TEAMS: u32 = 15;
const WATCHLIST_THRESHOLD: u32 = 3;

#[derive(Parser, Debug)]
#[command(name = "simulator")]
#[command(about = "Kafka Tutorial Classroom Simulator - Simulates all 15 teams")]
struct Args {
    /// Path to configuration file
    #[arg(short, long, default_value = "config.toml")]
    config: String,

    /// Override instances per team (for testing consumer group scaling)
    #[arg(long)]
    instances_per_team: Option<u32>,

    /// Quiet mode - minimal output (recommended when running with leaderboard TUI)
    #[arg(short, long)]
    quiet: bool,

    /// Log to file instead of stdout (recommended when running with leaderboard TUI)
    #[arg(long)]
    log_file: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Configure logging based on flags
    if args.quiet {
        // Quiet mode: only errors to stderr
        tracing_subscriber::registry()
            .with(tracing_subscriber::EnvFilter::new("error"))
            .with(tracing_subscriber::fmt::layer().with_writer(std::io::stderr))
            .init();
    } else if let Some(ref log_path) = args.log_file {
        // File logging mode
        let file = File::create(log_path)?;
        tracing_subscriber::registry()
            .with(
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| "info".into()),
            )
            .with(tracing_subscriber::fmt::layer().with_writer(file).with_ansi(false))
            .init();
    } else {
        // Default: log to stdout
        tracing_subscriber::registry()
            .with(
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| "info".into()),
            )
            .with(tracing_subscriber::fmt::layer())
            .init();
    }

    let mut settings = Settings::from_file(&args.config)?;

    // Override from CLI if provided
    if let Some(instances) = args.instances_per_team {
        settings.instances_per_team = instances;
    }

    info!(
        "Starting classroom simulator with {} teams, {} instance(s) per team",
        NUM_TEAMS, settings.instances_per_team
    );

    // Print startup message to stderr so it's visible even in quiet mode
    if args.quiet {
        eprintln!(
            "Simulator started: {} teams, {} instance(s) per team. Press Ctrl+C to stop.",
            NUM_TEAMS, settings.instances_per_team
        );
    }

    run_simulator(settings).await
}

async fn run_simulator(settings: Settings) -> Result<()> {
    // Create shared producer
    let producer: FutureProducer = kafka_common::kafka::new_sasl_config(
        &settings.brokers,
        &settings.username,
        &settings.password,
        &settings.security_protocol,
        &settings.sasl_mechanism,
    )
    .set("client.id", "classroom-simulator")
    .set("message.timeout.ms", "5000")
    .create()?;

    let producer = Arc::new(producer);
    info!("Connected to Kafka at {}", settings.brokers);

    let mut join_set = JoinSet::new();

    for team_number in 1..=NUM_TEAMS {
        // Shared watchlist state per team (across all instances)
        let watchlist = Arc::new(WatchlistState::new(WATCHLIST_THRESHOLD));

        for instance_id in 0..settings.instances_per_team {
            let consumer = create_consumer(&settings, team_number)?;
            let filter = get_filter(team_number);
            let producer_clone = Arc::clone(&producer);
            let watchlist_clone = Arc::clone(&watchlist);

            let simulator = TeamSimulator::new(
                team_number,
                instance_id,
                consumer,
                producer_clone,
                filter,
                watchlist_clone,
            );

            join_set.spawn(async move {
                if let Err(e) = simulator.run().await {
                    error!("Team {} instance {} failed: {}", team_number, instance_id, e);
                }
            });
        }
    }

    info!("All teams started. Press Ctrl+C to stop.");

    // Wait for Ctrl+C
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            info!("Received Ctrl+C, shutting down...");
        }
        _ = async {
            while join_set.join_next().await.is_some() {}
        } => {
            error!("All tasks completed unexpectedly");
        }
    }

    Ok(())
}

fn create_consumer(settings: &Settings, team_number: u32) -> Result<StreamConsumer> {
    let group_id = format!("team-{}", team_number);

    let consumer: StreamConsumer = kafka_common::kafka::new_sasl_config(
        &settings.brokers,
        &settings.username,
        &settings.password,
        &settings.security_protocol,
        &settings.sasl_mechanism,
    )
    .set("group.id", &group_id)
    .set("enable.auto.commit", "true")
    .set("auto.commit.interval.ms", "5000")
    .set("auto.offset.reset", "latest")
    .set("session.timeout.ms", "10000")
    .create()?;

    Ok(consumer)
}
