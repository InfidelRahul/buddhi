use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: Uuid,
    pub raw_input: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskContract {
    pub task_id: Uuid,
    pub task_type: TaskType,
    pub target_hints: Vec<String>,
    pub constraints: Vec<String>,
    pub risk_level: RiskLevel,
    pub privacy_level: PrivacyLevel,
    pub allowed_tools: Vec<String>,
    pub token_budget: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TaskType {
    BugFix,
    Refactor,
    Feature,
    TestGeneration,
    Explanation,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PrivacyLevel {
    Public,
    Internal,
    Sensitive,
    LocalOnly,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineEvent {
    pub session_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub event_type: EventType,
    pub payload: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventType {
    SessionStarted,
    TaskReceived,
    LocalBrainOptimized,
    CloudRequestSent,
    ToolExecuted,
    PatchApplied,
    ErrorOccurred,
}
