use crate::engine::InferenceEngine;
use dhi_core::error::{DhiError, Result};
use llama_cpp_2::context::params::LlamaContextParams;
use llama_cpp_2::llama_backend::LlamaBackend;
use llama_cpp_2::llama_batch::LlamaBatch;
use llama_cpp_2::model::params::LlamaModelParams;
use llama_cpp_2::model::AddBos;
use llama_cpp_2::model::LlamaModel;
use std::num::NonZeroU32;
use std::path::Path;

pub struct GgufEngine {
    backend: LlamaBackend,
    model: LlamaModel,
    n_ctx: u32,
}

impl GgufEngine {
    pub fn try_new(model_path: &Path, n_ctx: u32) -> Result<Self> {
        if !model_path.exists() {
            return Err(DhiError::Config(format!(
                "GGUF model not found: {}",
                model_path.display()
            )));
        }

        // Initialize backend globally for this engine instance
        let backend = LlamaBackend::init_num_threads(None, None);
        let params = LlamaModelParams::default();

        // Latest API requires &backend as first argument
        let model = LlamaModel::load_from_file(&backend, model_path, &params)
            .map_err(|e| DhiError::Config(format!("Failed to load GGUF model: {}", e)))?;

        Ok(Self {
            backend,
            model,
            n_ctx,
        })
    }
}

impl InferenceEngine for GgufEngine {
    fn engine_type(&self) -> &'static str {
        "gguf"
    }

    fn generate(&mut self, prompt: &str, max_tokens: usize) -> Result<String> {
        // BYPASS LIFETIME ISSUE: Create context dynamically per generation run.
        // This avoids needing self-referential struct crates like `ouroboros`.
        let ctx_params = LlamaContextParams::default().with_n_ctx(NonZeroU32::new(self.n_ctx));
        let mut ctx = self
            .model
            .new_context(&self.backend, ctx_params)
            .map_err(|e| DhiError::Config(format!("Context creation failed: {}", e)))?;

        let tokens = self
            .model
            .str_to_token(prompt, AddBos::Always)
            .map_err(|e| DhiError::Config(format!("Tokenization failed: {}", e)))?;

        // Latest API: LlamaBatch::new takes usize and returns Self directly (no Result)
        let mut batch = LlamaBatch::new(tokens.len() + max_tokens, 1);

        let last_index = tokens.len() as i32 - 1;
        for (i, token) in tokens.iter().enumerate() {
            let is_last = i == tokens.len() - 1;
            batch
                .add(*token, i as i32, &[0], is_last)
                .map_err(|e| DhiError::Config(format!("Batch add failed: {}", e)))?;
        }

        ctx.decode(&mut batch)
            .map_err(|e| DhiError::Config(format!("Prompt decode failed: {}", e)))?;

        let mut generated = String::new();

        for i in 0..max_tokens {
            let token = ctx
                .sample_token_greedy()
                .map_err(|e| DhiError::Config(format!("Sampling failed: {}", e)))?;

            if self.model.is_eog_token(token) {
                break;
            }

            // Latest API: token_to_str requires Special enum argument
            #[allow(deprecated)]
            let piece = self
                .model
                .token_to_str(token, llama_cpp_2::model::Special::Normal)
                .map_err(|e| DhiError::Config(format!("Detokenization failed: {}", e)))?;
            generated.push_str(&piece);

            let mut next_batch = LlamaBatch::new(1, 1);
            next_batch
                .add(token, last_index + 1 + i as i32, &[0], true)
                .map_err(|e| DhiError::Config(format!("Next batch add failed: {}", e)))?;

            ctx.decode(&mut next_batch)
                .map_err(|e| DhiError::Config(format!("Next decode failed: {}", e)))?;
        }

        Ok(generated)
    }
}
