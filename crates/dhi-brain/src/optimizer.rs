use crate::prompt::LocalBrainPromptBuilder;
use crate::types::{OptimizedIntent, RoutingDecision};
use dhi_core::error::{DhiError, Result};
use dhi_heuristics::types::HeuristicHints;
use std::time::Duration;
use tokio::time::timeout;

pub struct LocalBrainOptimizer {
    pub timeout: Duration,
    pub max_output_tokens: usize,
}

impl LocalBrainOptimizer {
    pub fn new(timeout_ms: u64, max_output_tokens: usize) -> Self {
        Self {
            timeout: Duration::from_millis(timeout_ms),
            max_output_tokens,
        }
    }

    pub async fn optimize(
        &self,
        raw_input: &str,
        hints: &HeuristicHints,
    ) -> Result<OptimizedIntent> {
        let prompt = LocalBrainPromptBuilder::build(raw_input, hints);
        tracing::debug!("Local brain prompt: {}", prompt);

        // Simulate local model inference with a timeout
        // In Phase 9, this will be replaced with actual Q4 model inference using candle/llama.cpp
        let result = timeout(self.timeout, self.simulate_inference(&prompt)).await;

        match result {
            Ok(Ok(intent)) => Ok(intent),
            Ok(Err(e)) => Err(e),
            Err(_) => {
                tracing::warn!("Local brain timed out. Falling back to heuristics.");
                self.fallback_to_heuristics(hints)
            }
        }
    }

    async fn simulate_inference(&self, _prompt: &str) -> Result<OptimizedIntent> {
        // Simulate network/model latency
        tokio::time::sleep(Duration::from_millis(50)).await;

        // Placeholder response
        Ok(OptimizedIntent {
            task_type: dhi_core::types::TaskType::BugFix,
            target_file_hints: vec!["src/main.rs".to_string()],
            target_symbol_hints: vec![],
            constraints: vec!["preserve_tests".to_string()],
            risk_level: dhi_core::types::RiskLevel::Medium,
            privacy_level: dhi_core::types::PrivacyLevel::Internal,
            routing_decision: RoutingDecision::Cloud,
            cloud_instruction_hint: "Fix the bug while preserving tests.".to_string(),
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
