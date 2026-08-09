use clap::Parser;
use dhi_config::loader::load_config;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "dhi", about = "Token-minimal AI coding engine")]
struct Args {
    /// Path to the configuration file
    #[arg(short, long, default_value = "config.yaml")]
    config: PathBuf,
}

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let args = Args::parse();
    let config = load_config(&args.config)?;
    
    tracing::info!("DHI Engine initialized successfully.");
    tracing::debug!("Loaded configuration: {:?}", config);

    Ok(())
}
