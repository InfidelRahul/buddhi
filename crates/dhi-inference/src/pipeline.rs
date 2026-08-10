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
}

impl InferencePipeline {
    pub fn try_new(model: LocalModel) -> Result<Self> {
        let device = Device::Cpu;
        let forward_pass = ForwardPass::new(&model.weights)?;

        // Skeleton: 4 layers
        let cache = KvCache::new(device.clone(), 4)
            .map_err(|e| DhiError::Config(format!("Failed to initialize KV cache: {}", e)))?;

        // Default to Greedy with temperature 1.0 for stable skeleton testing
        let sampler = Sampler::new(SamplingStrategy::Greedy, 1.0);

        Ok(Self {
            model,
            forward_pass,
            cache,
            sampler,
            device,
        })
    }

    pub fn generate(&mut self, prompt: &str, max_tokens: usize) -> Result<String> {
        let mut input_ids = self.model.tokenizer.encode(prompt)?;
        let mut generated_text = String::new();

        for _ in 0..max_tokens {
            let logits_vec = self.forward_pass.run(&input_ids, &mut self.cache)?;

            // Convert logits back to Tensor for the Sampler
            let vocab_size = logits_vec.len();
            let logits_tensor = Tensor::from_vec(logits_vec, &[1, vocab_size], &self.device)
                .map_err(|e| DhiError::Config(format!("Failed to create logits tensor: {}", e)))?;

            let next_token = self
                .sampler
                .sample(&logits_tensor)
                .map_err(|e| DhiError::Config(format!("Sampling failed: {}", e)))?;

            input_ids.push(next_token);

            let decoded = self.model.tokenizer.decode(&[next_token])?;
            generated_text.push_str(&decoded);

            // Simple EOS condition (token 0 is often EOS or padding in skeletons)
            if next_token == 0 {
                break;
            }
        }

        Ok(generated_text)
    }
}
