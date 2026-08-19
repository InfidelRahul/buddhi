use buddhi_brain::contract::IntentContractBuilder;
use buddhi_brain::optimizer::LocalBrainOptimizer;
use buddhi_config::loader::load_config;
use buddhi_core::session::Session;
use buddhi_engine::agent_loop::AgentLoop;
use buddhi_heuristics::parser::HeuristicParser;
use clap::Parser;
use std::io::{self, Write};
use std::path::PathBuf;

const ANSI_GREEN: &str = "\x1b[32m";
const ANSI_CYAN: &str = "\x1b[36m";
const ANSI_RESET: &str = "\x1b[0m";

#[derive(Parser, Debug)]
#[command(name = "buddhi", about = "Token-minimal AI coding engine")]
struct Args {
    #[arg(short, long, default_value = "config.yaml")]
    config: PathBuf,
    #[arg(short, long)]
    task: Option<String>,
    #[arg(long)]
    api_key: Option<String>,
    #[arg(long, default_value = "gpt-4o")]
    model: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let args = Args::parse();
    let config = load_config(&args.config)?;

    tracing::info!("Buddhi Engine initialized successfully.");

    if let Some(task_input) = args.task {
        tracing::info!("Processing task: {}", task_input);

        // Get API key from CLI arg or environment variable
        let api_key = args
            .api_key
            .or_else(|| std::env::var("OPENAI_API_KEY").ok())
            .expect("OpenAI API key required. Set OPENAI_API_KEY env var or use --api-key");

        // Run the autonomous agent loop
        let agent = AgentLoop::new(std::env::current_dir()?, api_key, args.model);

        match agent.run(&task_input).await {
            Ok(_) => {
                println!(
                    "{}✅ Task completed successfully!{}",
                    ANSI_GREEN, ANSI_RESET
                );
            }
            Err(e) => {
                eprintln!("Task failed: {}", e);
                std::process::exit(1);
            }
        }
    } else {
        tracing::warn!("No task provided. Use --task to specify a task.");
    }

    Ok(())
}
