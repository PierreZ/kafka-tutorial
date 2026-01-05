mod config;
mod kafka;
mod state;
mod ui;
mod validation;

/// Number of teams in the tutorial (team-1 through team-15)
pub const NUM_TEAMS: u32 = 15;

use anyhow::Result;
use clap::Parser;
use std::fs::File;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Parser, Debug)]
#[command(name = "leaderboard")]
#[command(about = "Kafka Tutorial Leaderboard - Real-time scoring and achievement tracking")]
struct Args {
    /// Path to configuration file
    #[arg(short, long, default_value = "config.toml")]
    config: String,

    /// Run in demo mode (no Kafka required)
    #[arg(long)]
    demo: bool,

    /// Log file path (logs to file to avoid breaking TUI)
    #[arg(long, default_value = "leaderboard.log")]
    log_file: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Initialize tracing to file (stdout breaks TUI)
    let log_file = File::create(&args.log_file)?;
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .with(tracing_subscriber::fmt::layer().with_writer(log_file).with_ansi(false))
        .init();

    if args.demo {
        info!("Running in demo mode (no Kafka connection)");
        return ui::run_demo().await;
    }

    info!("Loading configuration from: {}", args.config);

    let settings = config::Settings::from(&args.config)?;
    info!("Configuration loaded successfully");

    // Run the application
    ui::run(settings).await
}
