use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub local_brain: LocalBrainConfig,
    pub cloud: CloudConfig,
    pub budget: BudgetConfig,
    pub security: SecurityConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalBrainConfig {
    pub enabled: bool,
    pub model_path: PathBuf,
    pub max_output_tokens: usize,
    pub timeout_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudConfig {
    pub provider: String,
    pub model: String,
    pub api_key_env: String,
    pub max_output_tokens: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetConfig {
    pub max_tokens_per_turn: usize,
    pub max_total_tokens_per_task: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub sandbox_enabled: bool,
    pub secret_scanning_enabled: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            local_brain: LocalBrainConfig {
                enabled: true,
                model_path: PathBuf::from("models/qwen2.5-coder-instruct-q4.gguf"),
                max_output_tokens: 120,
                timeout_ms: 800,
            },
            cloud: CloudConfig {
                provider: "openai".to_string(),
                model: "gpt-4o".to_string(),
                api_key_env: "OPENAI_API_KEY".to_string(),
                max_output_tokens: 1024,
            },
            budget: BudgetConfig {
                max_tokens_per_turn: 4096,
                max_total_tokens_per_task: 16384,
            },
            security: SecurityConfig {
                sandbox_enabled: true,
                secret_scanning_enabled: true,
            },
        }
    }
}
