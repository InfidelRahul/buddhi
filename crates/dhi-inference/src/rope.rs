use candle_core::{Result as CandleResult, Tensor};

pub struct RotaryEmbedding {
    cos_cache: Tensor,
    sin_cache: Tensor,
}

impl RotaryEmbedding {
    pub fn new(dim: usize, max_seq_len: usize, device: &candle_core::Device) -> CandleResult<Self> {
        // Generate position indices [0, 1, ..., max_seq_len-1]
        let pos =
            Tensor::arange(0u32, max_seq_len as u32, device)?.to_dtype(candle_core::DType::F32)?;

        // Compute inverse frequencies: 1 / (10000^(2i/dim))
        let inv_freq = Tensor::arange(0u32, (dim / 2) as u32, device)?
            .to_dtype(candle_core::DType::F32)?
            .affine(2.0 / dim as f64, 0.0)?
            .exp()?;

        // Outer product: [seq_len, dim/2]
        let freqs = pos.unsqueeze(1)?.matmul(&inv_freq.unsqueeze(0)?)?;

        // Concatenate to get [seq_len, dim]
        let freqs = Tensor::cat(&[&freqs, &freqs], 1)?;

        Ok(Self {
            cos_cache: freqs.cos()?,
            sin_cache: freqs.sin()?,
        })
    }

    pub fn apply(&self, x: &Tensor, offset: usize) -> CandleResult<Tensor> {
        // x shape: [batch, seq_len, num_heads, head_dim]
        let (_b, seq_len, _h, head_dim) = x.dims4()?;

        let cos = self
            .cos_cache
            .narrow(0, offset, seq_len)?
            .unsqueeze(0)?
            .unsqueeze(2)?;
        let sin = self
            .sin_cache
            .narrow(0, offset, seq_len)?
            .unsqueeze(0)?
            .unsqueeze(2)?;

        // Rotate half: [-x[..., dim/2:], x[..., :dim/2]]
        let half = head_dim / 2;
        let x1 = x.narrow(3, 0, half)?;
        let x2 = x.narrow(3, half, half)?;
        let rotated = Tensor::cat(&[&x2.neg()?, &x1], 3)?;

        // Apply RoPE: x * cos + rotate_half(x) * sin
        x.broadcast_mul(&cos)? + rotated.broadcast_mul(&sin)?
    }
}
