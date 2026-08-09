use crate::registry::ToolRegistry;
use crate::types::ToolResult;
use dhi_core::error::{DhiError, Result};
use dhi_core::types::ToolCall;

pub struct ToolExecutor {
    registry: ToolRegistry,
}

impl ToolExecutor {
    pub fn new(registry: ToolRegistry) -> Self {
        Self { registry }
    }

    pub async fn execute(&self, call: &ToolCall) -> Result<ToolResult> {
        let tool = self
            .registry
            .get(&call.name)
            .ok_or_else(|| DhiError::ToolExecution(format!("Tool not found: {}", call.name)))?;

        tool.execute(call.arguments.clone()).await
    }
}
