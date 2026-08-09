use crate::registry::ToolRegistry;
use crate::types::ToolResult;
use dhi_core::error::{DhiError, Result};
use dhi_core::types::ToolCall;
use std::path::{Path, PathBuf};

pub struct ToolExecutor {
    registry: ToolRegistry,
    project_root: PathBuf,
}

impl ToolExecutor {
    pub fn new(registry: ToolRegistry, project_root: PathBuf) -> Self {
        Self {
            registry,
            project_root,
        }
    }

    pub async fn execute(&self, call: &ToolCall) -> Result<ToolResult> {
        let tool = self
            .registry
            .get(&call.name)
            .ok_or_else(|| DhiError::ToolExecution(format!("Tool not found: {}", call.name)))?;

        tool.execute(call.arguments.clone(), &self.project_root)
            .await
    }
}
