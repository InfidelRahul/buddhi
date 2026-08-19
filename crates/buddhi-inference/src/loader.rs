use crate::engine::InferenceEngine;
use crate::gguf_engine::GgufEngine;
use crate::safetensors_engine::SafetensorsEngine;
use buddhi_core::error::{DhiError, Result};
use std::path::Path;

pub struct ModelLoader;

impl ModelLoader {
    pub fn load(
        model_path: &Path,
        tokenizer_path: &Path,
        n_ctx: u32,
    ) -> Result<Box<dyn InferenceEngine>> {
        // Route 1: GGUF (llama.cpp backend)
        if model_path.extension().and_then(|e| e.to_str()) == Some("gguf") {
            tracing::info!("Routing to GGUF engine (llama-cpp-2)...");
            let engine = GgufEngine::try_new(model_path, n_ctx)?;
            return Ok(Box::new(engine));
        }

        // Route 2: Safetensors Directory (candle-core backend)
        if model_path.is_dir() {
            tracing::info!("Routing to Safetensors engine (candle-core)...");
            let engine = SafetensorsEngine::try_new(model_path, tokenizer_path)?;
            return Ok(Box::new(engine));
        }

        Err(DhiError::Config(format!(
            "Unsupported model format: {}. Provide a .gguf file or a directory containing .safetensors.",
            model_path.display()
        )))
    }
}
