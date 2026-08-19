use buddhi_core::types::TaskType;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeuristicHints {
    pub detected_files: Vec<String>,
    pub detected_symbols: Vec<String>,
    pub detected_task_type: Option<TaskType>,
    pub detected_constraints: Vec<String>,
}
