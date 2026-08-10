use crate::tokenizer::LocalTokenizer;
use crate::weights::ModelWeights;
use dhi_core::error::{DhiError, Result};
use std::path::Path;

pub struct LocalModel {
    pub weights: ModelWeights,
    pub tokenizer: LocalTokenizer,
}

pub struct ModelLoader;

impl ModelLoader {
    pub fn load(model_path: &Path, tokenizer_path: &Path) -> Result<LocalModel> {
        if !model_path.exists() {
            return Err(DhiError::Config(format!(
                "Model file not found: {}",
                model_path.display()
            )));
        }
        if !tokenizer_path.exists() {
            return Err(DhiError::Config(format!(
                "Tokenizer file not found: {}",
                tokenizer_path.display()
            )));
        }

        let weights = ModelWeights::load(model_path, &candle_core::Device::Cpu)?;
        let tokenizer = LocalTokenizer::load(tokenizer_path)?;

        Ok(LocalModel { weights, tokenizer })
    }
}
