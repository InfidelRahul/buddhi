use dhi_core::error::{DhiError, Result};
use serde::Deserialize;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize, Clone)]
pub struct ModelConfig {
    pub hidden_size: usize,
    pub num_attention_heads: usize,
    pub num_hidden_layers: usize,
    pub intermediate_size: usize,
    pub vocab_size: usize,
    #[serde(default = "default_rms_norm_eps")]
    pub rms_norm_eps: f64,
}

fn default_rms_norm_eps() -> f64 {
    1e-6
}

impl ModelConfig {
    pub fn load(model_dir: &Path) -> Result<Self> {
        let config_path = model_dir.join("config.json");
        if !config_path.exists() {
            return Err(DhiError::Config(format!(
                "config.json not found in {}",
                model_dir.display()
            )));
        }

        let content = fs::read_to_string(&config_path)
            .map_err(|e| DhiError::Config(format!("Failed to read config.json: {}", e)))?;

        let config: ModelConfig = serde_json::from_str(&content)
            .map_err(|e| DhiError::Config(format!("Failed to parse config.json: {}", e)))?;

        Ok(config)
    }
}
