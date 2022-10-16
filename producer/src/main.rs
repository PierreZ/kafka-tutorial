use producer::{produce, Settings};
use std::path::PathBuf;
use structopt::StructOpt;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

#[derive(StructOpt, Debug)]
#[structopt(name = "producer")]
struct Opt {
    /// config file
    #[structopt(name = "config", short = "c", parse(from_os_str))]
    pub config_file: PathBuf,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env())
        .init();

    let opt = Opt::from_args();

    let settings = Settings::from(opt.config_file.to_str().unwrap())?;

    produce(&settings).await;

    Ok(())
}
