use crate::engine::InferenceEngine;
use buddhi_core::error::{BuddhiError, Result};
use candle_core::Device;
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
            return Err(BuddhiError::Config(format!(
                "Model dir not found: {}",
                model_dir.display()
            )));
        }
        if !tokenizer_path.exists() {
            return Err(BuddhiError::Config(format!(
                "Tokenizer not found: {}",
                tokenizer_path.display()
            )));
        }
        let tokenizer = Tokenizer::from_file(tokenizer_path)
            .map_err(|e| BuddhiError::Config(format!("Failed to load tokenizer: {}", e)))?;
        Ok(Self {
            device: Device::cpu(),
            tokenizer,
        })
    }
}
impl InferenceEngine for SafetensorsEngine {
    fn engine_type(&self) -> &'static str {
        "safetensors"
    }
    fn generate(&mut self, prompt: &str, max_tokens: usize) -> Result<String> {
        self.generate_stream(prompt, max_tokens, &mut |_| {})
    }
    fn generate_stream(
        &mut self,
        prompt: &str,
        _max_tokens: usize,
        on_token: &mut (dyn FnMut(&str) + Send),
    ) -> Result<String> {
        let dummy_output = format!(
            "[Safetensors] Processed: {}",
            prompt.chars().take(20).collect::<String>()
        );
        on_token(&dummy_output);
        Ok(dummy_output)
    }
}
