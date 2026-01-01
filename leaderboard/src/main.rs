mod config;
mod kafka;
mod state;
mod ui;
mod validation;

/// Number of teams in the tutorial (team-1 through team-15)
pub const NUM_TEAMS: u32 = 15;

use anyhow::Result;
use clap::Parser;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Parser, Debug)]
#[command(name = "leaderboard")]
#[command(about = "Kafka Tutorial Leaderboard - Real-time scoring and achievement tracking")]
struct Args {
    /// Path to configuration file
    #[arg(short, long, default_value = "config.toml")]
    config: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "leaderboard=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let args = Args::parse();
    info!("Loading configuration from: {}", args.config);

    let settings = config::Settings::from(&args.config)?;
    info!("Configuration loaded successfully");

    // Run the application
    ui::run(settings).await
}
