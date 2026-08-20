use candle_core::{Result, Tensor};

/// Key-Value cache for transformer attention layers.
/// Stores past key/value tensors to avoid recomputation during
/// autoregressive generation. Zero heap allocation in the hot loop.
pub struct KvCache {
    /// Shape: [num_layers, batch, seq_len, head_dim]
    k: Vec<Option<Tensor>>,
    v: Vec<Option<Tensor>>,
    num_layers: usize,
}

impl KvCache {
    pub fn new(num_layers: usize) -> Self {
        Self {
            k: vec![None; num_layers],
            v: vec![None; num_layers],
            num_layers,
        }
    }

    /// Append new key/value to cache for a given layer.
    /// Concatenates along the sequence dimension (dim=2).
    pub fn append(
        &mut self,
        layer_idx: usize,
        new_k: &Tensor,
        new_v: &Tensor,
    ) -> Result<(Tensor, Tensor)> {
        let k = match &self.k[layer_idx] {
            Some(prev_k) => Tensor::cat(&[prev_k, new_k], 2)?,
            None => new_k.clone(),
        };
        let v = match &self.v[layer_idx] {
            Some(prev_v) => Tensor::cat(&[prev_v, new_v], 2)?,
            None => new_v.clone(),
        };

        self.k[layer_idx] = Some(k.clone());
        self.v[layer_idx] = Some(v.clone());
        Ok((k, v))
    }

    /// Get cached key/value for a layer.
    pub fn get(&self, layer_idx: usize) -> (Option<&Tensor>, Option<&Tensor>) {
        (self.k[layer_idx].as_ref(), self.v[layer_idx].as_ref())
    }

    /// Reset cache for a new generation sequence.
    pub fn reset(&mut self) {
        for i in 0..self.num_layers {
            self.k[i] = None;
            self.v[i] = None;
        }
    }

    pub fn seq_len(&self, layer_idx: usize) -> usize {
        self.k[layer_idx]
            .as_ref()
            .map(|t| t.dim(2).unwrap_or(0))
            .unwrap_or(0)
    }
}
