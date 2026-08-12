use crate::engine::InferenceEngine;
use dhi_core::error::{DhiError, Result};
use llama_cpp_2::context::LlamaContext;
use llama_cpp_2::llama_backend::LlamaBackend;
use llama_cpp_2::llama_batch::LlamaBatch;
use llama_cpp_2::model::params::LlamaModelParams;
use llama_cpp_2::model::AddBos;
use llama_cpp_2::model::LlamaModel;
use std::path::Path;
use std::sync::Arc;

pub struct GgufEngine {
    model: Arc<LlamaModel>,
    ctx: LlamaContext,
}

impl GgufEngine {
    pub fn try_new(model_path: &Path, n_ctx: u32) -> Result<Self> {
        if !model_path.exists() {
            return Err(DhiError::Config(format!(
                "GGUF model not found: {}",
                model_path.display()
            )));
        }

        LlamaBackend::init();
        let params = LlamaModelParams::default();
        let model = LlamaModel::load_from_file(model_path, params)
            .map_err(|e| DhiError::Config(format!("Failed to load GGUF model: {}", e)))?;

        let ctx = model
            .new_context(n_ctx, 512, 0)
            .map_err(|e| DhiError::Config(format!("Failed to create llama context: {}", e)))?;

        Ok(Self {
            model: Arc::new(model),
            ctx,
        })
    }
}

impl InferenceEngine for GgufEngine {
    fn engine_type(&self) -> &'static str {
        "gguf"
    }

    fn generate(&mut self, prompt: &str, max_tokens: usize) -> Result<String> {
        let tokens = self
            .model
            .str_to_token(prompt, AddBos::Always)
            .map_err(|e| DhiError::Config(format!("Tokenization failed: {}", e)))?;

        let mut batch = LlamaBatch::new(tokens.len() as i32 + max_tokens as i32, 1)
            .map_err(|e| DhiError::Config(format!("Batch creation failed: {}", e)))?;

        let last_index = (tokens.len() - 1) as i32;
        for (i, token) in tokens.iter().enumerate() {
            let is_last = i == tokens.len() - 1;
            batch
                .add(*token, i as i32, &[0], is_last)
                .map_err(|e| DhiError::Config(format!("Batch add failed: {}", e)))?;
        }

        self.ctx
            .decode(&mut batch)
            .map_err(|e| DhiError::Config(format!("Prompt decode failed: {}", e)))?;

        let mut generated = String::new();
        for i in 0..max_tokens {
            let token = self
                .ctx
                .sample_token_greedy()
                .map_err(|e| DhiError::Config(format!("Sampling failed: {}", e)))?;

            if self.model.is_eog_token(token) {
                break;
            }

            let piece = self
                .model
                .token_to_str(token)
                .map_err(|e| DhiError::Config(format!("Detokenization failed: {}", e)))?;
            generated.push_str(&piece);

            let mut next_batch = LlamaBatch::new(1, 1)
                .map_err(|e| DhiError::Config(format!("Next batch creation failed: {}", e)))?;
            next_batch
                .add(token, last_index + 1 + i as i32, &[0], true)
                .map_err(|e| DhiError::Config(format!("Next batch add failed: {}", e)))?;

            self.ctx
                .decode(&mut next_batch)
                .map_err(|e| DhiError::Config(format!("Next decode failed: {}", e)))?;
        }

        Ok(generated)
    }
}
