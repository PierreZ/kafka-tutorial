use producer::{produce, Settings};
use simple_logger::SimpleLogger;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "producer")]
struct Opt {
    /// config file
    #[structopt(name = "config", short = "c", parse(from_os_str))]
    pub config_file: PathBuf,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    SimpleLogger::new().init().unwrap();
    let opt = Opt::from_args();

    let settings = Settings::from(opt.config_file.to_str().unwrap())?;

    produce(&settings).await;

    Ok(())
}
