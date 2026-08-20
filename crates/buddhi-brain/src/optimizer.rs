use crate::intent::OptimizedIntent;
use buddhi_core::context::ContextManager;
use buddhi_core::error::Result;
use buddhi_inference::{InferenceEngine, LocalInferenceEngine};
use std::sync::Arc;
use tokio::sync::Mutex;

/// The Optimizer decides whether to use local inference or cloud LLM.
/// It attempts local generation first for speed, falling back to cloud
/// if local inference is unavailable or fails.
pub struct Optimizer {
    pipeline: Option<Arc<Mutex<Box<dyn InferenceEngine>>>>,
}

impl Optimizer {
    pub fn new() -> Self {
        // Initialize local inference engine

        let engine = LocalInferenceEngine::new();

        Self {
            pipeline: Some(Arc::new(Mutex::new(
                Box::new(engine) as Box<dyn InferenceEngine>
            ))),
        }
    }

    /// Optimize the user's intent by attempting local inference first.
    pub async fn optimize(&self, prompt: &str, cm: &mut ContextManager) -> Result<OptimizedIntent> {
        // Try local inference first
        if let Some(pipeline) = &self.pipeline {
            let p = pipeline.lock().await;
            match p.generate(prompt, 120) {
                Ok(output) => {
                    cm.add_message(&output).ok();
                    return self.parse_llm_output(&output);
                }
                Err(e) => {
                    tracing::warn!("Local generation failed: {}", e);
                }
            }
        }

        // Fallback: return unoptimized intent
        Ok(OptimizedIntent {
            intent: prompt.to_string(),
            confidence: 0.5,
        })
    }

    fn parse_llm_output(&self, output: &str) -> Result<OptimizedIntent> {
        // Parse the LLM output into an OptimizedIntent
        // For now, return a basic intent
        Ok(OptimizedIntent {
            intent: output.to_string(),
            confidence: 0.8,
        })
    }
}

impl Default for Optimizer {
    fn default() -> Self {
        Self::new()
    }
}
