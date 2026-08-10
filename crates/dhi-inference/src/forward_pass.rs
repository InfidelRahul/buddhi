use crate::kv_cache::KvCache;
use crate::transformer::TransformerBlock;
use crate::weights::ModelWeights;
use candle_core::{DType, Device, Tensor};
use candle_nn::{Module, VarBuilder};
use dhi_core::error::{DhiError, Result};

pub struct ForwardPass {
    blocks: Vec<TransformerBlock>,
    embed_tokens: candle_nn::Embedding,
    norm: candle_nn::RmsNorm,
    lm_head: candle_nn::Linear,
    device: Device,
}

impl ForwardPass {
    pub fn new(weights: &ModelWeights) -> Result<Self> {
        let device = Device::Cpu;
        let hidden_size = 1024; // Qwen 0.5B hidden size
        let num_layers = 4; // Skeleton layer count
        let num_heads = 16; // Qwen 0.5B num_heads
        let vocab_size = 32000;

        // Create VarBuilder from loaded tensors (candle 0.8 takes ownership of HashMap)
        let vb = VarBuilder::from_tensors(weights.tensors.clone(), DType::F32, &device);

        let embed_tokens =
            candle_nn::embedding(vocab_size, hidden_size, vb.pp("model.embed_tokens"))
                .map_err(|e| DhiError::Config(format!("Failed to load embeddings: {}", e)))?;

        let norm = candle_nn::rms_norm(hidden_size, 1e-6, vb.pp("model.norm"))
            .map_err(|e| DhiError::Config(format!("Failed to load final norm: {}", e)))?;

        let lm_head = candle_nn::linear(hidden_size, vocab_size, vb.pp("lm_head"))
            .map_err(|e| DhiError::Config(format!("Failed to load lm_head: {}", e)))?;

        let mut blocks = Vec::with_capacity(num_layers);
        for i in 0..num_layers {
            let block = TransformerBlock::load(&vb, i, hidden_size, num_heads)
                .map_err(|e| DhiError::Config(format!("Failed to load block {}: {}", i, e)))?;
            blocks.push(block);
        }

        Ok(Self {
            blocks,
            embed_tokens,
            norm,
            lm_head,
            device,
        })
    }

    pub fn run(&self, input_ids: &[u32], cache: &mut KvCache) -> Result<Vec<f32>> {
        let seq_len = input_ids.len();

        // Convert input_ids to tensor
        let input_tensor = Tensor::from_vec(input_ids.to_vec(), &[1, seq_len], &self.device)
            .map_err(|e| DhiError::Config(format!("Failed to create input tensor: {}", e)))?;

        // 1. Embedding Lookup
        let mut x = self
            .embed_tokens
            .forward(&input_tensor)
            .map_err(|e| DhiError::Config(format!("Embedding lookup failed: {}", e)))?;

        // 2. Pass through all Transformer Blocks
        for block in &self.blocks {
            x = block
                .forward(&x, cache)
                .map_err(|e| DhiError::Config(format!("Block forward failed: {}", e)))?;
        }

        // 3. Final Norm & LM Head
        let x = self
            .norm
            .forward(&x)
            .map_err(|e| DhiError::Config(format!("Final norm failed: {}", e)))?;

        let logits = self
            .lm_head
            .forward(&x)
            .map_err(|e| DhiError::Config(format!("LM head failed: {}", e)))?;

        // Extract logits for the last token
        let last_logits = logits.get(0)?.get(seq_len - 1)?;
        let logits_vec: Vec<f32> = last_logits
            .to_vec1()
            .map_err(|e| DhiError::Config(format!("Failed to convert logits: {}", e)))?;

        Ok(logits_vec)
    }
}
