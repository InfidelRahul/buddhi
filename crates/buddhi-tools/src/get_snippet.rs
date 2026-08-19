use crate::types::{Tool, ToolResult};
use buddhi_core::error::{DhiError, Result};
use buddhi_security::path_guard::PathGuard;
use std::fs;
use std::path::Path;

pub struct GetSnippetTool;

#[async_trait::async_trait]
impl Tool for GetSnippetTool {
    fn name(&self) -> &str {
        "get_snippet"
    }

    async fn execute(&self, args: serde_json::Value, project_root: &Path) -> Result<ToolResult> {
        let path = args
            .get("path")
            .and_then(|p| p.as_str())
            .ok_or_else(|| DhiError::ToolExecution("Missing path argument".to_string()))?;
        let start = args
            .get("start")
            .and_then(|s| s.as_u64())
            .ok_or_else(|| DhiError::ToolExecution("Missing start argument".to_string()))?
            as usize;
        let end = args
            .get("end")
            .and_then(|e| e.as_u64())
            .ok_or_else(|| DhiError::ToolExecution("Missing end argument".to_string()))?
            as usize;

        let safe_path = PathGuard::validate(path, project_root)?;
        let content =
            fs::read_to_string(safe_path).map_err(|e| DhiError::ToolExecution(e.to_string()))?;
        let lines: Vec<&str> = content.lines().collect();

        if start >= end || start >= lines.len() {
            return Err(DhiError::ToolExecution("Invalid line range".to_string()));
        }

        let snippet: Vec<&str> = lines[start..end.min(lines.len())].to_vec();
        Ok(ToolResult {
            success: true,
            output: snippet.join("\n"),
            token_cost: 0,
        })
    }
}
