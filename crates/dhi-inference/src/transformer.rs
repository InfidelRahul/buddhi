use crate::kv_cache::KvCache;
use candle_core::{Result as CandleResult, Tensor, D};
use candle_nn::{Linear, RmsNorm, VarBuilder};

pub struct TransformerBlock {
    attention_norm: RmsNorm,
    mlp_norm: RmsNorm,
    // Attention layers (Q, K, V, O)
    wq: Linear,
    wk: Linear,
    wv: Linear,
    wo: Linear,
    // MLP layers (SwiGLU: gate, up, down)
    w_gate: Linear,
    w_up: Linear,
    w_down: Linear,
    pub layer_id: usize,
}

impl TransformerBlock {
    pub fn load(vb: &VarBuilder, layer_id: usize, hidden_size: usize) -> CandleResult<Self> {
        let prefix = format!("model.layers.{}", layer_id);

        // Load norms
        let attention_norm = candle_nn::rms_norm(
            hidden_size,
            1e-6,
            vb.pp(&format!("{}.input_layernorm", prefix)),
        )?;
        let mlp_norm = candle_nn::rms_norm(
            hidden_size,
            1e-6,
            vb.pp(&format!("{}.post_attention_layernorm", prefix)),
        )?;

        // Load attention projections
        let wq = candle_nn::linear(
            hidden_size,
            hidden_size,
            vb.pp(&format!("{}.self_attn.q_proj", prefix)),
        )?;
        let wk = candle_nn::linear(
            hidden_size,
            hidden_size,
            vb.pp(&format!("{}.self_attn.k_proj", prefix)),
        )?;
        let wv = candle_nn::linear(
            hidden_size,
            hidden_size,
            vb.pp(&format!("{}.self_attn.v_proj", prefix)),
        )?;
        let wo = candle_nn::linear(
            hidden_size,
            hidden_size,
            vb.pp(&format!("{}.self_attn.o_proj", prefix)),
        )?;

        // Load MLP projections
        let w_gate = candle_nn::linear(
            hidden_size,
            hidden_size * 4,
            vb.pp(&format!("{}.mlp.gate_proj", prefix)),
        )?;
        let w_up = candle_nn::linear(
            hidden_size,
            hidden_size * 4,
            vb.pp(&format!("{}.mlp.up_proj", prefix)),
        )?;
        let w_down = candle_nn::linear(
            hidden_size * 4,
            hidden_size,
            vb.pp(&format!("{}.mlp.down_proj", prefix)),
        )?;

        Ok(Self {
            attention_norm,
            mlp_norm,
            wq,
            wk,
            wv,
            wo,
            w_gate,
            w_up,
            w_down,
            layer_id,
        })
    }

    pub fn forward(&self, x: &Tensor, _cache: &mut KvCache) -> CandleResult<Tensor> {
        // 1. Attention Sublayer (Pre-Norm)
        let norm_x = self.attention_norm.forward(x)?;

        // Skeleton attention computation
        let q = self.wq.forward(&norm_x)?;
        let k = self.wk.forward(&norm_x)?;
        let v = self.wv.forward(&norm_x)?;
        let attn_out = self.wo.forward(&v)?; // Simplified: bypass attention math for now

        let x = (x + attn_out)?;

        // 2. MLP Sublayer (Pre-Norm, SwiGLU)
        let norm_x = self.mlp_norm.forward(&x)?;
        let gate = self.w_gate.forward(&norm_x)?.silu()?;
        let up = self.w_up.forward(&norm_x)?;
        let mlp_out = self.w_down.forward(&(gate * up)?)?;

        let x = (x + mlp_out)?;

        Ok(x)
    }
}
