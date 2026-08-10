use crate::prompt::LocalBrainPromptBuilder;
use crate::types::{OptimizedIntent, RoutingDecision};
use dhi_core::error::{DhiError, Result};
use dhi_heuristics::types::HeuristicHints;
use dhi_inference::loader::{LocalModel, ModelLoader};
use dhi_inference::pipeline::InferencePipeline;
use std::path::PathBuf;
use std::time::Duration;
use tokio::time::timeout;

pub struct LocalBrainOptimizer {
    pub timeout: Duration,
    pub max_output_tokens: usize,
    pipeline: Option<InferencePipeline>,
}

impl LocalBrainOptimizer {
    pub fn try_new(
        timeout_ms: u64,
        max_output_tokens: usize,
        model_path: PathBuf,
        tokenizer_path: PathBuf,
    ) -> Result<Self> {
        let pipeline = match ModelLoader::load(&model_path, &tokenizer_path) {
            Ok(model) => Some(InferencePipeline::try_new(model)?),
            Err(e) => {
                tracing::warn!(
                    "Failed to load local model: {}. Falling back to heuristics.",
                    e
                );
                None
            }
        };

        Ok(Self {
            timeout: Duration::from_millis(timeout_ms),
            max_output_tokens,
            pipeline,
        })
    }

    pub async fn optimize(
        &self,
        raw_input: &str,
        hints: &HeuristicHints,
    ) -> Result<OptimizedIntent> {
        let prompt = LocalBrainPromptBuilder::build(raw_input, hints);
        tracing::debug!("Local brain prompt: {}", prompt);

        if let Some(pipeline) = &self.pipeline {
            // Clone pipeline for async task (InferencePipeline needs to be mutable,
            // so we wrap in Arc<Mutex> or spawn a blocking task)
            let mut pipeline = pipeline.clone(); // Requires Clone impl or Arc<Mutex>

            let result = timeout(
                self.timeout,
                tokio::task::spawn_blocking(move || pipeline.generate(&prompt, 120)),
            )
            .await;

            match result {
                Ok(Ok(Ok(output))) => {
                    return self.parse_llm_output(&output);
                }
                Ok(Ok(Err(e))) => tracing::warn!("Local generation failed: {}", e),
                Ok(Err(e)) => tracing::warn!("Local task panicked: {}", e),
                Err(_) => tracing::warn!("Local brain timed out."),
            }
        }

        self.fallback_to_heuristics(hints)
    }

    fn parse_llm_output(&self, output: &str) -> Result<OptimizedIntent> {
        // Skeleton: Parse JSON from LLM output
        // In a real implementation, we would extract the JSON block
        tracing::debug!("LLM output: {}", output);

        // Placeholder parsed intent
        Ok(OptimizedIntent {
            task_type: dhi_core::types::TaskType::BugFix,
            target_file_hints: vec!["src/main.rs".to_string()],
            target_symbol_hints: vec![],
            constraints: vec!["preserve_tests".to_string()],
            risk_level: dhi_core::types::RiskLevel::Medium,
            privacy_level: dhi_core::types::PrivacyLevel::Internal,
            routing_decision: RoutingDecision::Cloud,
            cloud_instruction_hint: output.chars().take(100).collect(),
        })
    }

    fn fallback_to_heuristics(&self, hints: &HeuristicHints) -> Result<OptimizedIntent> {
        let task_type = match &hints.detected_task_type {
            Some(t) => t.clone(),
            None => dhi_core::types::TaskType::Unknown,
        };

        Ok(OptimizedIntent {
            task_type,
            target_file_hints: hints.detected_files.clone(),
            target_symbol_hints: hints.detected_symbols.clone(),
            constraints: hints.detected_constraints.clone(),
            risk_level: dhi_core::types::RiskLevel::Medium,
            privacy_level: dhi_core::types::PrivacyLevel::Internal,
            routing_decision: RoutingDecision::Cloud,
            cloud_instruction_hint: "Optimize prompt for cloud model".to_string(),
        })
    }
}
