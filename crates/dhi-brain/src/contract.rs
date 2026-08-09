use crate::types::OptimizedIntent;
use dhi_core::types::TaskContract;
use uuid::Uuid;

pub struct IntentContractBuilder;

impl IntentContractBuilder {
    pub fn build(task_id: Uuid, intent: &OptimizedIntent, token_budget: usize) -> TaskContract {
        TaskContract {
            task_id,
            task_type: intent.task_type.clone(),
            target_hints: intent.target_file_hints.clone(),
            constraints: intent.constraints.clone(),
            risk_level: intent.risk_level.clone(),
            privacy_level: intent.privacy_level.clone(),
            allowed_tools: vec![
                "expand".to_string(),
                "get_snippet".to_string(),
                "get_symbol".to_string(),
                "search".to_string(),
                "replace".to_string(),
                "run_tests".to_string(),
                "run_linter".to_string(),
            ],
            token_budget,
        }
    }
}
