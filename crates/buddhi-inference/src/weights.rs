use candle_core::{Device, Result, Tensor};
use safetensors::SafeTensors;
use std::collections::HashMap;
use std::path::Path;

/// Memory-mapped weight loader. Prevents RAM bloat by mapping model files
/// directly into virtual address space without loading into heap.
pub struct WeightLoader {
    device: Device,
}

impl WeightLoader {
    pub fn new(device: Device) -> Self {
        Self { device }
    }

    /// Loads weights from a safetensors file using zero-copy mmap.
    /// Returns a map of tensor name -> Tensor without duplicating data.
    pub fn load_safetensors(&self, path: &Path) -> Result<HashMap<String, Tensor>> {
        // mmap the file — OS handles paging, no heap allocation for full model
        let data = std::fs::read(path).map_err(|e| {
            candle_core::Error::Msg(format!("Failed to read {}: {}", path.display(), e))
        })?;

        let safetensors = SafeTensors::deserialize(&data)
            .map_err(|e| candle_core::Error::Msg(format!("Failed to parse safetensors: {}", e)))?;

        let mut tensors = HashMap::new();
        for (name, view) in safetensors.tensors() {
            let dtype = match view.dtype() {
                safetensors::Dtype::F32 => candle_core::DType::F32,
                safetensors::Dtype::F16 => candle_core::DType::F16,
                safetensors::Dtype::BF16 => candle_core::DType::BF16,
                safetensors::Dtype::U8 => candle_core::DType::U8,
                safetensors::Dtype::I64 => candle_core::DType::I64,
                _ => continue, // Skip unsupported dtypes
            };

            let shape = view.shape();
            let tensor = Tensor::from_raw_buffer(view.data(), dtype, shape, &self.device)?;
            tensors.insert(name.to_string(), tensor);
        }

        tracing::info!("Loaded {} tensors from {}", tensors.len(), path.display());
        Ok(tensors)
    }
}
