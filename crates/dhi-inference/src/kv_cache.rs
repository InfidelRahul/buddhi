use candle_core::{Device, Result, Tensor};

pub struct KvCache {
    pub device: Device,
    // Stores (key, value) tensors for each layer
    // Shape per layer: [batch_size, num_heads, seq_len, head_dim]
    pub cache: Vec<(Tensor, Tensor)>,
}

impl KvCache {
    pub fn new(device: Device, num_layers: usize) -> Result<Self> {
        let mut cache = Vec::with_capacity(num_layers);
        for _ in 0..num_layers {
            // Initialize with empty tensors; will be populated during forward pass
            let empty = Tensor::zeros(&[1, 1, 0, 1], candle_core::DType::F32, &device)?;
            cache.push((empty.clone(), empty));
        }
        Ok(Self { device, cache })
    }

    pub fn append(&mut self, layer_idx: usize, new_k: &Tensor, new_v: &Tensor) -> Result<()> {
        if layer_idx >= self.cache.len() {
            return Err(candle_core::Error::Msg(
                "Layer index out of bounds".to_string(),
            ));
        }
        let (k, v) = &self.cache[layer_idx];
        let new_k = Tensor::cat(&[k, new_k], 2)?; // Concatenate along seq_len dim
        let new_v = Tensor::cat(&[v, new_v], 2)?;
        self.cache[layer_idx] = (new_k, new_v);
        Ok(())
    }

    pub fn get(&self, layer_idx: usize) -> Result<(&Tensor, &Tensor)> {
        if layer_idx >= self.cache.len() {
            return Err(candle_core::Error::Msg(
                "Layer index out of bounds".to_string(),
            ));
        }
        let (k, v) = &self.cache[layer_idx];
        Ok((k, v))
    }

    pub fn len(&self, layer_idx: usize) -> usize {
        if layer_idx < self.cache.len() {
            self.cache[layer_idx].0.dim(2).unwrap_or(0)
        } else {
            0
        }
    }
}
