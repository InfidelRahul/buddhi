use crate::engine::InferenceEngine;
use buddhi_core::error::{DhiError, Result};
use llama_cpp_2::context::params::LlamaContextParams;
use llama_cpp_2::llama_backend::LlamaBackend;
use llama_cpp_2::llama_batch::LlamaBatch;
use llama_cpp_2::model::params::LlamaModelParams;
use llama_cpp_2::model::AddBos;
use llama_cpp_2::model::LlamaModel;
use llama_cpp_2::sampling::LlamaSampler;
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
        let backend = LlamaBackend::init()
            .map_err(|e| DhiError::Config(format!("Backend init failed: {}", e)))?;
        let params = LlamaModelParams::default();
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
        self.generate_stream(prompt, max_tokens, &mut |_| {})
    }
    fn generate_stream(
        &mut self,
        prompt: &str,
        max_tokens: usize,
        on_token: &mut (dyn FnMut(&str) + Send),
    ) -> Result<String> {
        let ctx_params = LlamaContextParams::default().with_n_ctx(NonZeroU32::new(self.n_ctx));
        let mut ctx = self
            .model
            .new_context(&self.backend, ctx_params)
            .map_err(|e| DhiError::Config(format!("Context creation failed: {}", e)))?;
        let tokens = self
            .model
            .str_to_token(prompt, AddBos::Always)
            .map_err(|e| DhiError::Config(format!("Tokenization failed: {}", e)))?;
        let mut batch = LlamaBatch::new(tokens.len() + max_tokens, 1);
        for (i, token) in tokens.iter().enumerate() {
            let is_last = i == tokens.len() - 1;
            batch
                .add(*token, i as i32, &[0], is_last)
                .map_err(|e| DhiError::Config(format!("Batch add failed: {}", e)))?;
        }
        ctx.decode(&mut batch)
            .map_err(|e| DhiError::Config(format!("Prompt decode failed: {}", e)))?;
        let mut generated = String::new();
        let mut sampler = LlamaSampler::chain_simple([LlamaSampler::greedy()]);
        for n_cur in tokens.len() as i32..tokens.len() as i32 + max_tokens as i32 {
            let token = sampler.sample(&ctx, batch.n_tokens() - 1);
            sampler.accept(token);
            if self.model.is_eog_token(token) {
                break;
            }
            #[allow(deprecated)]
            let piece = self
                .model
                .token_to_str(token, llama_cpp_2::model::Special::Plaintext)
                .map_err(|e| DhiError::Config(format!("Detokenization failed: {}", e)))?;
            on_token(&piece);
            generated.push_str(&piece);
            let mut next_batch = LlamaBatch::new(1, 1);
            next_batch
                .add(token, n_cur, &[0], true)
                .map_err(|e| DhiError::Config(format!("Next batch add failed: {}", e)))?;
            ctx.decode(&mut next_batch)
                .map_err(|e| DhiError::Config(format!("Next decode failed: {}", e)))?;
            batch = next_batch;
        }
        Ok(generated)
    }
}
