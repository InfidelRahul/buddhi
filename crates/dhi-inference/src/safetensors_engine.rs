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
        self.generate_stream(prompt, max_tokens, |_| {})
    }

    fn generate_stream<F>(
        &mut self,
        prompt: &str,
        _max_tokens: usize,
        mut on_token: F,
    ) -> Result<String>
    where
        F: FnMut(&str),
    {
        let dummy_output = format!(
            "[Safetensors] Processed: {}",
            prompt.chars().take(20).collect::<String>()
        );
        on_token(&dummy_output);
        Ok(dummy_output)
    }
}
