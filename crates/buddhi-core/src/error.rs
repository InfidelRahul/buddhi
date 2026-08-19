use thiserror::Error;

#[derive(Error, Debug)]
pub enum DhiError {
    #[error("Configuration error: {0}")]
    Config(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("Model error: {0}")]
    Model(String),
    #[error("Tool execution failed: {0}")]
    ToolExecution(String),
    #[error("Budget exceeded: {0}")]
    BudgetExceeded(String),
}

pub type Result<T> = std::result::Result<T, DhiError>;
