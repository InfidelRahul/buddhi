use crate::tokenizer::BuddhiTokenizer;
use crate::weights::WeightLoader;
use buddhi_core::error::{BuddhiError, Result};
use candle_core::Device;
use std::path::Path;

/// Trait defining the inference engine interface.
pub trait InferenceEngine: Send + Sync {
    fn load_weights(&mut self, path: &Path) -> Result<()>;
    fn load_tokenizer(&mut self, path: &Path) -> Result<()>;
    fn generate(&self, prompt: &str, max_tokens: usize) -> Result<String>;
}

/// Determines the best available hardware device at compile/runtime.
/// Falls back to CPU if no GPU feature is enabled or available.
pub fn get_best_device() -> Device {
    #[cfg(feature = "cuda")]
    {
        match Device::cuda_if_available(0) {
            Ok(device) => {
                tracing::info!("Accelerator: CUDA device selected.");
                return device;
            }
            Err(e) => tracing::warn!("CUDA requested but unavailable: {}. Falling back.", e),
        }
    }

    #[cfg(feature = "metal")]
    {
        match Device::new_metal(0) {
            Ok(device) => {
                tracing::info!("Accelerator: Apple Metal device selected.");
                return device;
            }
            Err(e) => tracing::warn!("Metal requested but unavailable: {}. Falling back.", e),
        }
    }

    tracing::info!("Accelerator: Falling back to CPU.");
    Device::Cpu
}

/// Local inference engine implementation using candle.
pub struct LocalInferenceEngine {
    device: Device,
    tokenizer: Option<BuddhiTokenizer>,
    weights_loaded: bool,
    _num_layers: usize,
}

impl LocalInferenceEngine {
    /// Instantiates the engine, automatically selecting the best hardware device.
    pub fn new() -> Self {
        Self {
            device: get_best_device(),
            tokenizer: None,
            weights_loaded: false,
            _num_layers: 32,
        }
    }
}

impl Default for LocalInferenceEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl InferenceEngine for LocalInferenceEngine {
    fn load_weights(&mut self, path: &Path) -> Result<()> {
        let loader = WeightLoader::new(self.device.clone());
        let _tensors = loader
            .load_safetensors(path)
            .map_err(|e| BuddhiError::Config(format!("Failed to load weights: {}", e)))?;
        self.weights_loaded = true;
        tracing::info!("Model weights loaded onto {:?}.", self.device);
        Ok(())
    }

    fn load_tokenizer(&mut self, path: &Path) -> Result<()> {
        self.tokenizer = Some(BuddhiTokenizer::from_file(path)?);
        tracing::info!("Tokenizer loaded successfully.");
        Ok(())
    }

    fn generate(&self, prompt: &str, _max_tokens: usize) -> Result<String> {
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
        Ok(prompt.to_string())
    }
}
