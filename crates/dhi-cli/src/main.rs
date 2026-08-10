use clap::Parser;
use dhi_brain::contract::IntentContractBuilder;
use dhi_brain::optimizer::LocalBrainOptimizer;
use dhi_config::loader::load_config;
use dhi_core::session::Session;
use dhi_heuristics::parser::HeuristicParser;
use std::io::{self, Write};
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

        let model_path = PathBuf::from(&config.local_brain.model_path);
        let tokenizer_path = PathBuf::from("tokenizer.json");

        let brain_optimizer = LocalBrainOptimizer::try_new(
            config.local_brain.timeout_ms,
            config.local_brain.max_output_tokens,
            model_path,
            tokenizer_path,
        )?;

        // 2. Create session
        let mut session = Session::new(std::env::current_dir()?, std::env::current_dir()?);

        // 3. Start task
        let task_id = session.start_task(task_input.clone())?;

        // 4. Run heuristics
        let hints = heuristic_parser.parse(&task_input);

        // 5. Run local brain optimization
        let intent = brain_optimizer.optimize(&task_input, &hints).await?;
        tracing::info!("Optimized intent: {:?}", intent);

        // 6. Build and set contract
        let contract =
            IntentContractBuilder::build(task_id, &intent, config.budget.max_tokens_per_turn);
        session.set_contract(contract.clone());

        // 7. Stream output simulation (Phase 24 UX)
        println!("\n--- DHI Local Brain Output ---");
        let stdout = io::stdout();
        let mut lock = stdout.lock();

        // In a real implementation, this would call pipeline.generate_stream
        // For now, we simulate streaming the cloud_instruction_hint
        let output = &intent.cloud_instruction_hint;
        for word in output.split_whitespace() {
            write!(lock, "{} ", word)?;
            lock.flush()?;
            std::thread::sleep(std::time::Duration::from_millis(50)); // Simulate latency
        }
        println!("\n------------------------------\n");
    } else {
        tracing::warn!("No task provided. Use --task to specify a task.");
    }

    Ok(())
}
