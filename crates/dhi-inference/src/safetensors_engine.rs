use crate::engine::InferenceEngine;
use candle_core::Device;
use dhi_core::error::{DhiError, Result};
use std::path::Path;
use tokenizers::Tokenizer;

pub struct SafetensorsEngine {
    #[allow(dead_code)]
    device: Device,
    #[allow(dead_code)]
    tokenizer: Tokenizer,
}

impl SafetensorsEngine {
    pub fn try_new(model_dir: &Path, tokenizer_path: &Path) -> Result<Self> {
        if !model_dir.exists() {
            return Err(DhiError::Config(format!(
                "Model dir not found: {}",
                model_dir.display()
            )));
        }
        if !tokenizer_path.exists() {
            return Err(DhiError::Config(format!(
                "Tokenizer not found: {}",
                tokenizer_path.display()
            )));
        }

        let tokenizer = Tokenizer::from_file(tokenizer_path)
            .map_err(|e| DhiError::Config(format!("Failed to load tokenizer: {}", e)))?;

        tracing::info!("Safetensors engine initialized (Candle-core backend)");

        Ok(Self {
            device: Device::Cpu,
            tokenizer,
        })
    }
}

impl InferenceEngine for SafetensorsEngine {
    fn engine_type(&self) -> &'static str {
        "safetensors"
    }

    fn generate(&mut self, prompt: &str, max_tokens: usize) -> Result<String> {
        // Skeleton: Actual candle-transformers forward pass will be wired in Phase 31
        tracing::debug!(
            "Safetensors generating {} tokens for: {}",
            max_tokens,
            prompt
        );
        Ok(format!(
            "[Safetensors Engine] Processed prompt: {}",
            prompt.chars().take(50).collect::<String>()
        ))
    }
}
