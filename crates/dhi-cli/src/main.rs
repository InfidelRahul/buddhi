use clap::Parser;
use dhi_brain::contract::IntentContractBuilder;
use dhi_brain::optimizer::LocalBrainOptimizer;
use dhi_config::loader::load_config;
use dhi_core::session::Session;
use dhi_engine::prompt::CloudPromptBuilder;
use dhi_engine::r#loop::AgentLoop;
use dhi_heuristics::parser::HeuristicParser;
use dhi_llm::openai::OpenAiClient;
use dhi_token::budget::TokenBudget;
use dhi_token::counter::CharCounter;
use std::path::PathBuf;
use std::sync::Arc;

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
        session.set_contract(contract.clone());

        // 7. Initialize Engine Components
        let counter = Arc::new(CharCounter);
        let budget = TokenBudget::new(counter, config.budget.max_tokens_per_turn);

        let api_key = std::env::var(&config.cloud.api_key_env).unwrap_or_default();
        let provider = OpenAiClient::new(
            api_key,
            "https://api.openai.com/v1".to_string(),
            config.cloud.model.clone(),
        );

        // 8. Build Prompt and Run Agent Loop
        let rules = dhi_rules::loader::RuleLoader::load(&session.project_root)?;
        let memory = dhi_memory::store::MemoryStore::load(&session.project_root)?;
        let context = "fn main() { println!(\"Hello, world!\"); }"; // Placeholder context

        let prompt = CloudPromptBuilder::build(&contract, &rules, &memory, context);

        let mut agent_loop = AgentLoop::new(&provider, budget);
        agent_loop.run(&contract, &prompt).await?;

        tracing::info!("Agent loop completed successfully.");
    } else {
        tracing::warn!("No task provided. Use --task to specify a task.");
    }

    Ok(())
}
