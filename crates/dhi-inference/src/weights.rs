use candle_core::{Device, Tensor};
use dhi_core::error::{DhiError, Result};
use std::collections::HashMap;
use std::path::Path;

pub struct ModelWeights {
    pub tensors: HashMap<String, Tensor>,
}

impl ModelWeights {
    pub fn load(model_path: &Path, device: &Device) -> Result<Self> {
        if !model_path.exists() {
            return Err(DhiError::Config(format!(
                "Model file not found: {}",
                model_path.display()
            )));
        }

        // Load safetensors file into a HashMap of tensors
        let tensors = candle_core::safetensors::load(model_path, device)
            .map_err(|e| DhiError::Config(format!("Failed to load safetensors: {}", e)))?;

        Ok(Self { tensors })
    }

    pub fn get(&self, name: &str) -> Result<&Tensor> {
        self.tensors
            .get(name)
            .ok_or_else(|| DhiError::Config(format!("Weight key not found: {}", name)))
    }
}
