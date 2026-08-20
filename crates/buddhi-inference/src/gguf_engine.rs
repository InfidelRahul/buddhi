use crate::engine::InferenceEngine;
use buddhi_core::error::{BuddhiError, Result};
use llama_cpp_2::context::params::LlamaContextParams;
use llama_cpp_2::llama_backend::LlamaBackend;
use llama_cpp_2::llama_batch::LlamaBatch;
use llama_cpp_2::model::params::LlamaModelParams;
use llama_cpp_2::model::{AddBos, LlamaModel};
use llama_cpp_2::token::LlamaToken;
use std::path::Path;
use std::sync::Arc;

/// GGUF-based inference engine for fast local intent routing.
/// Optimized for reasoning models like Qwen3.8 that emit <think> blocks.
pub struct GgufEngine {
    backend: LlamaBackend,
    model: Arc<LlamaModel>,
}

impl GgufEngine {
    /// Load a GGUF model from disk.
    pub fn new(model_path: &Path) -> Result<Self> {
        let backend = LlamaBackend::init();
        let model_params = LlamaModelParams::default();
        let model = LlamaModel::from_file(model_path.to_str().unwrap_or(""), model_params)
            .map_err(|e| BuddhiError::Config(format!("Failed to load GGUF model: {}", e)))?;

        tracing::info!("GGUF model loaded: {}", model_path.display());
        Ok(Self {
            backend,
            model: Arc::new(model),
        })
    }

    /// Strips <think>...</think> reasoning blocks from Qwen3.8 output.
    fn strip_think_blocks(text: &str) -> String {
        let mut result = String::with_capacity(text.len());
        let mut in_think = false;
        let mut chars = text.chars().peekable();

        while let Some(c) = chars.next() {
            if c == '<' {
                let mut tag = String::from("<");
                while let Some(&next) = chars.peek() {
                    tag.push(next);
                    chars.next();
                    if next == '>' {
                        break;
                    }
                }
                if tag == "<think>" {
                    in_think = true;
                    continue;
                }
                if tag == "</think>" {
                    in_think = false;
                    continue;
                }
                result.push_str(&tag);
            } else if !in_think {
                result.push(c);
            }
        }
        result.trim().to_string()
    }
}

impl InferenceEngine for GgufEngine {
    fn load_weights(&mut self, _path: &Path) -> Result<()> {
        Ok(())
    }
    fn load_tokenizer(&mut self, _path: &Path) -> Result<()> {
        Ok(())
    }

    fn generate(&self, prompt: &str, max_tokens: usize) -> Result<String> {
        let ctx_params = LlamaContextParams::default();
        let mut ctx = self
            .model
            .new_context(ctx_params)
            .map_err(|e| BuddhiError::Config(format!("Context creation failed: {}", e)))?;

        let tokens = self
            .model
            .str_to_token(prompt, AddBos::Always)
            .map_err(|e| BuddhiError::Config(format!("Tokenization failed: {}", e)))?;

        let mut batch = LlamaBatch::new(2048, 1);
        for (i, token) in tokens.iter().enumerate() {
            let is_last = i == tokens.len() - 1;
            batch
                .add_token(*token, i as i32, 0, is_last)
                .map_err(|e| BuddhiError::Config(format!("Batch add failed: {}", e)))?;
        }

        ctx.decode(&mut batch)
            .map_err(|e| BuddhiError::Config(format!("Decode failed: {}", e)))?;

        // Simplified greedy decoding for intent routing
        let mut output = String::new();
        let mut last_token = tokens.last().copied().unwrap_or(LlamaToken(0));

        for _ in 0..max_tokens {
            let logits = ctx.get_logits();
            let next_token_idx = logits
                .iter()
                .enumerate()
                .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
                .map(|(i, _)| i as i32)
                .unwrap_or(0);

            let next_token = LlamaToken(next_token_idx);
            if next_token == self.model.token_eos() {
                break;
            }

            let piece = self
                .model
                .token_to_str(next_token)
                .map_err(|e| BuddhiError::Config(format!("Detokenize failed: {}", e)))?;
            output.push_str(&piece);
            last_token = next_token;

            batch.clear();
            batch
                .add_token(next_token, tokens.len() as i32, 0, true)
                .map_err(|e| BuddhiError::Config(format!("Batch add failed: {}", e)))?;
            ctx.decode(&mut batch)
                .map_err(|e| BuddhiError::Config(format!("Decode failed: {}", e)))?;
        }

        Ok(Self::strip_think_blocks(&output))
    }
}
