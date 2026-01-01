pub mod app;
mod widgets;

use crate::config::Settings;
use anyhow::Result;

pub async fn run(settings: Settings) -> Result<()> {
    app::run(settings).await
}

pub async fn run_demo() -> Result<()> {
    app::run_demo().await
}
