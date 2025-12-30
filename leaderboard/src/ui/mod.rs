pub mod app;
mod widgets;

use anyhow::Result;
use crate::config::Settings;

pub async fn run(settings: Settings) -> Result<()> {
    app::run(settings).await
}
