use clap::Parser;
use dhi_brain::contract::IntentContractBuilder;
use dhi_brain::optimizer::LocalBrainOptimizer;
use dhi_config::loader::load_config;
use dhi_core::session::Session;
use dhi_heuristics::parser::HeuristicParser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "dhi", about = "Token-minimal AI coding engine")]
struct Args {
    /// Path to the configuration file
    #[arg(short, long, default_value = "config.yaml")]
    config: PathBuf,

    /// The task to execute
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

        // 1. Initialize components
        let heuristic_parser = HeuristicParser::try_new()?;
        let brain_optimizer = LocalBrainOptimizer::new(
            config.local_brain.timeout_ms,
            config.local_brain.max_output_tokens,
        );

        // 2. Create session
        let mut session = Session::new(std::env::current_dir()?, std::env::current_dir()?);

        // 3. Start task
        let task_id = session.start_task(task_input.clone())?;
        tracing::debug!("Task started with ID: {}", task_id);

        // 4. Run heuristics
        let hints = heuristic_parser.parse(&task_input);
        tracing::debug!("Heuristic hints: {:?}", hints);

        // 5. Run local brain optimization
        let intent = brain_optimizer.optimize(&task_input, &hints).await?;
        tracing::debug!("Optimized intent: {:?}", intent);

        // 6. Build and set contract
        let contract =
            IntentContractBuilder::build(task_id, &intent, config.budget.max_tokens_per_turn);
        session.set_contract(contract);

        tracing::info!("Task contract built and set in session.");
    } else {
        tracing::warn!("No task provided. Use --task to specify a task.");
    }

    Ok(())
}
