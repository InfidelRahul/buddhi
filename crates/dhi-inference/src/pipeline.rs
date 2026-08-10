use crate::context::ContextManager;
use crate::forward_pass::ForwardPass;
use crate::kv_cache::KvCache;
use crate::loader::LocalModel;
use crate::sampler::{Sampler, SamplingStrategy};
use candle_core::{Device, Tensor};
use dhi_core::error::{DhiError, Result};

pub struct InferencePipeline {
    model: LocalModel,
    forward_pass: ForwardPass,
    cache: KvCache,
    sampler: Sampler,
    device: Device,
    context_manager: ContextManager,
}

impl InferencePipeline {
    pub fn try_new(model: LocalModel, max_context_tokens: usize) -> Result<Self> {
        let device = Device::Cpu;
        let forward_pass = ForwardPass::new(&model.weights)?;

        // Skeleton: 4 layers
        let cache = KvCache::new(device.clone(), 4)
            .map_err(|e| DhiError::Config(format!("Failed to initialize KV cache: {}", e)))?;

        // Default to Greedy with temperature 1.0 for stable skeleton testing
        let sampler = Sampler::new(SamplingStrategy::Greedy, 1.0);
        let context_manager = ContextManager::new(max_context_tokens);

        Ok(Self {
            model,
            forward_pass,
            cache,
            sampler,
            device,
            context_manager,
        })
    }

    pub fn generate_stream<F>(
        &mut self,
        prompt: &str,
        max_tokens: usize,
        mut on_token: F,
    ) -> Result<String>
    where
        F: FnMut(&str),
    {
        // Add prompt to context manager
        self.context_manager.add_message(prompt)?;

        // Get full context for generation
        let full_context = self.context_manager.get_context();

        let mut input_ids = self.model.tokenizer.encode(&full_context)?;
        let mut generated_text = String::new();

        for _ in 0..max_tokens {
            let logits_vec = self.forward_pass.run(&input_ids, &mut self.cache)?;

            let vocab_size = logits_vec.len();
            let logits_tensor = Tensor::from_vec(logits_vec, &[1, vocab_size], &self.device)
                .map_err(|e| DhiError::Config(format!("Failed to create logits tensor: {}", e)))?;

            let next_token = self
                .sampler
                .sample(&logits_tensor)
                .map_err(|e| DhiError::Config(format!("Sampling failed: {}", e)))?;

            input_ids.push(next_token);

            let decoded = self.model.tokenizer.decode(&[next_token])?;

            // Stream the token to the callback
            on_token(&decoded);
            generated_text.push_str(&decoded);

            // Simple EOS condition
            if next_token == 0 {
                break;
            }
        }

        // Add generated response to context for multi-turn support
        self.context_manager.add_message(&generated_text)?;

        Ok(generated_text)
    }

    pub fn generate(&mut self, prompt: &str, max_tokens: usize) -> Result<String> {
        self.generate_stream(prompt, max_tokens, |_| {})
    }
}
