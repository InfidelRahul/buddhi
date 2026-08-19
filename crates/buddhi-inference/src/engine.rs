use crate::kv_cache::KvCache;
use crate::tokenizer::BuddhiTokenizer;
use crate::weights::WeightLoader;
use buddhi_core::error::{BuddhiError, Result};
use candle_core::Device;
use std::path::Path;

/// Top-level inference engine. Orchestrates weights, tokenizer, KV cache,
/// and forward pass into a single generate() call.
pub struct InferenceEngine {
    device: Device,
    tokenizer: Option<BuddhiTokenizer>,
    weights_loaded: bool,
    num_layers: usize,
}

impl InferenceEngine {
    pub fn new(device: Device) -> Self {
        Self {
            device,
            tokenizer: None,
            weights_loaded: false,
            num_layers: 32, // Default; overridden by model config
        }
    }

    /// Load model weights from a safetensors file.
    pub fn load_weights(&mut self, path: &Path) -> Result<()> {
        let loader = WeightLoader::new(self.device.clone());
        let _tensors = loader.load_safetensors(path)?;
        self.weights_loaded = true;
        tracing::info!("Model weights loaded successfully.");
        Ok(())
    }

    /// Load tokenizer from file.
    pub fn load_tokenizer(&mut self, path: &Path) -> Result<()> {
        self.tokenizer = Some(BuddhiTokenizer::from_file(path)?);
        tracing::info!("Tokenizer loaded successfully.");
        Ok(())
    }

    /// Generate text from a prompt.
    pub fn generate(&self, prompt: &str, max_tokens: usize) -> Result<String> {
        if !self.weights_loaded {
            return Err(BuddhiError::Config("Weights not loaded.".into()));
        }
        let tokenizer = self
            .tokenizer
            .as_ref()
            .ok_or_else(|| BuddhiError::Config("Tokenizer not loaded.".into()))?;

        let token_ids = tokenizer.encode(prompt)?;
        tracing::info!("Prompt encoded: {} tokens", token_ids.len());

        // TODO: Implement autoregressive generation loop with KV cache
        // For now, return the prompt as-is
        Ok(prompt.to_string())
    }
}
