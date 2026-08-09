use crate::kv_cache::KvCache;
use candle_core::{Result as CandleResult, Tensor};

pub struct TransformerBlock {
    // In Phase 20, these will hold actual candle_nn::Linear and RmsNorm layers
    // mapped from safetensors weights.
    pub layer_id: usize,
}

impl TransformerBlock {
    pub fn new(layer_id: usize) -> Self {
        Self { layer_id }
    }

    pub fn forward(&self, x: &Tensor, _cache: &mut KvCache) -> CandleResult<Tensor> {
        // Pre-Norm Transformer Architecture (Standard for Llama/Qwen)
        // 1. Attention Sublayer: x = x + Attention(RmsNorm(x))
        // 2. MLP Sublayer:       x = x + MLP(RmsNorm(x))

        // Skeleton: Pass tensor through unchanged until weights are loaded
        Ok(x.clone())
    }
}
