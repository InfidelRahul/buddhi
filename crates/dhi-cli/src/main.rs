use clap::Parser;
use dhi_brain::contract::IntentContractBuilder;
use dhi_brain::optimizer::LocalBrainOptimizer;
use dhi_config::loader::load_config;
use dhi_core::session::Session;
use dhi_heuristics::parser::HeuristicParser;
use std::io::{self, Write};
use std::path::PathBuf;

const ANSI_GREEN: &str = "\x1b[32m";
const ANSI_CYAN: &str = "\x1b[36m";
const ANSI_RESET: &str = "\x1b[0m";

#[derive(Parser, Debug)]
#[command(name = "dhi", about = "Token-minimal AI coding engine")]
struct Args {
    #[arg(short, long, default_value = "config.yaml")]
    config: PathBuf,

    #[arg(short, long)]
    task: Option<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let args = Args::parse();
    let config = load_config(&args.config)?;

    tracing::info!("DHI Engine initialized successfully.");

    if let Some(task_input) = args.task {
        tracing::info!("Processing task: {}", task_input);

        let heuristic_parser = HeuristicParser::try_new()?;
        let model_path = PathBuf::from(&config.local_brain.model_path);
        let tokenizer_path = PathBuf::from("tokenizer.json");

        let brain_optimizer = LocalBrainOptimizer::try_new(
            config.local_brain.timeout_ms,
            config.local_brain.max_output_tokens,
            model_path,
            tokenizer_path,
        )?;

        let mut session = Session::new(std::env::current_dir()?, std::env::current_dir()?);
        let task_id = session.start_task(task_input.clone())?;
        let hints = heuristic_parser.parse(&task_input);

        println!(
            "\n{}--- DHI Local Brain Streaming ---{}",
            ANSI_CYAN, ANSI_RESET
        );

        // Pass the ANSI-colored streaming callback to the optimizer
        // We use print! and flush() to avoid capturing StdoutLock which is !Send
        let intent = brain_optimizer
            .optimize(&task_input, &hints, move |token| {
                print!("{}{}{}", ANSI_GREEN, token, ANSI_RESET);
                io::stdout().flush().unwrap();
            })
            .await?;

        println!(
            "\n{}-------------------------------{}\n",
            ANSI_CYAN, ANSI_RESET
        );

        tracing::info!("Optimized intent: {:?}", intent);

        let contract =
            IntentContractBuilder::build(task_id, &intent, config.budget.max_tokens_per_turn);
        session.set_contract(contract.clone());
    } else {
        tracing::warn!("No task provided. Use --task to specify a task.");
    }

    Ok(())
}
