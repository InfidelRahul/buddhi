use serde::{Deserialize, Serialize};
use dhi_core::types::{TaskType, RiskLevel, PrivacyLevel};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizedIntent {
    pub task_type: TaskType,
    pub target_file_hints: Vec<String>,
    pub target_symbol_hints: Vec<String>,
    pub constraints: Vec<String>,
    pub risk_level: RiskLevel,
    pub privacy_level: PrivacyLevel,
    pub routing_decision: RoutingDecision,
    pub cloud_instruction_hint: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RoutingDecision {
    LocalOnly,
    Cloud,
    Hybrid,
}
