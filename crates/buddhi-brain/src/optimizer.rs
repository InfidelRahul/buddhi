use crate::intent::OptimizedIntent;
use buddhi_core::context::ContextManager;
use buddhi_core::error::Result;
use buddhi_inference::{GgufEngine, InferenceEngine};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

/// The Optimizer acts as the Local Scout, using a fast 2B GGUF model
/// to strip reasoning blocks and route intent before calling the Cloud Brain.
pub struct Optimizer {
    scout: Option<Arc<Mutex<Box<dyn InferenceEngine>>>>,
}

impl Optimizer {
    pub fn new() -> Self {
        // Attempt to load the local GGUF scout model
        let model_path = PathBuf::from(".buddhi/models/qwen3.8-2b-q4_k_m.gguf");
        let scout = if model_path.exists() {
            match GgufEngine::new(&model_path) {
                Ok(engine) => {
                    tracing::info!("Local Scout initialized with Qwen3.8-2B.");
                    Some(Arc::new(Mutex::new(
                        Box::new(engine) as Box<dyn InferenceEngine>
                    )))
                }
                Err(e) => {
                    tracing::warn!("Failed to load local scout model: {}", e);
                    None
                }
            }
        } else {
            tracing::info!(
                "No local scout model found at {}. Using Cloud Brain only.",
                model_path.display()
            );
            None
        };

        Self { scout }
    }

    /// Routes the prompt through the Local Scout first, then to the Cloud Brain.
    pub async fn optimize(&self, prompt: &str, cm: &mut ContextManager) -> Result<OptimizedIntent> {
        let mut clean_prompt = prompt.to_string();

        // Phase 1: Local Scout strips <think> blocks and refines intent
        if let Some(scout) = &self.scout {
            let engine = scout.lock().await;
            match engine.generate(prompt, 64) {
                Ok(routed_intent) => {
                    tracing::info!("Scout routed intent: {}", routed_intent);
                    clean_prompt = format!("{}\n\nRefined Intent: {}", prompt, routed_intent);
                    cm.add_message(&format!("[Scout]: {}", routed_intent)).ok();
                }
                Err(e) => tracing::warn!("Scout failed, falling back to raw prompt: {}", e),
            }
        }

        // Phase 2: Pass refined prompt to Cloud Brain (handled by AgentLoop)
        Ok(OptimizedIntent {
            intent: clean_prompt,
            confidence: if self.scout.is_some() { 0.95 } else { 0.5 },
        })
    }
}

impl Default for Optimizer {
    fn default() -> Self {
        Self::new()
    }
}
