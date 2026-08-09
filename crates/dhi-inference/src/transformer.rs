use crate::kv_cache::KvCache;
use crate::rope::RotaryEmbedding;
use candle_core::{Result as CandleResult, Tensor, D};
use candle_nn::{Linear, RmsNorm, VarBuilder};

pub struct TransformerBlock {
    attention_norm: RmsNorm,
    mlp_norm: RmsNorm,
    wq: Linear,
    wk: Linear,
    wv: Linear,
    wo: Linear,
    w_gate: Linear,
    w_up: Linear,
    w_down: Linear,
    rope: RotaryEmbedding,
    pub layer_id: usize,
}

impl TransformerBlock {
    pub fn load(
        vb: &VarBuilder,
        layer_id: usize,
        hidden_size: usize,
        num_heads: usize,
    ) -> CandleResult<Self> {
        let prefix = format!("model.layers.{}", layer_id);
        let head_dim = hidden_size / num_heads;
        let device = vb.device();

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

        let rope = RotaryEmbedding::new(head_dim, 4096, device)?;

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
            rope,
            layer_id,
        })
    }

    pub fn forward(&self, x: &Tensor, cache: &mut KvCache) -> CandleResult<Tensor> {
        let (b, seq_len, hidden_size) = x.dims3()?;
        let num_heads = 16; // Qwen 0.5B num_heads
        let head_dim = hidden_size / num_heads;

        // 1. Attention Sublayer
        let norm_x = self.attention_norm.forward(x)?;

        let q = self
            .wq
            .forward(&norm_x)?
            .reshape((b, seq_len, num_heads, head_dim))?
            .transpose(1, 2)?;
        let k = self
            .wk
            .forward(&norm_x)?
            .reshape((b, seq_len, num_heads, head_dim))?
            .transpose(1, 2)?;
        let v = self
            .wv
            .forward(&norm_x)?
            .reshape((b, seq_len, num_heads, head_dim))?
            .transpose(1, 2)?;

        // Apply RoPE (offset based on cache length)
        let offset = cache.len(self.layer_id);
        let q = self.rope.apply(&q, offset)?;
        let k = self.rope.apply(&k, offset)?;

        // Update KV Cache
        cache.append(self.layer_id, &k, &v)?;

        // Retrieve full K and V from cache
        let (full_k, full_v) = cache.get(self.layer_id)?;

        // Scaled Dot-Product Attention
        let scale = (head_dim as f64).sqrt();
        let attn_weights = q
            .matmul(&full_k.transpose(2, 3)?)?
            .affine(1.0 / scale, 0.0)?;
        let attn_weights = candle_nn::ops::softmax(&attn_weights, D::Minus1)?;
        let attn_out = attn_weights.matmul(&full_v)?;

        let attn_out = attn_out
            .transpose(1, 2)?
            .reshape((b, seq_len, hidden_size))?;
        let attn_out = self.wo.forward(&attn_out)?;

        let x = (x + attn_out)?;

        // 2. MLP Sublayer
        let norm_x = self.mlp_norm.forward(&x)?;
        let gate = self.w_gate.forward(&norm_x)?.silu()?;
        let up = self.w_up.forward(&norm_x)?;
        let mlp_out = self.w_down.forward(&(gate * up)?)?;

        let x = (x + mlp_out)?;

        Ok(x)
    }
}
