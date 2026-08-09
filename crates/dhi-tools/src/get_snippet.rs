use crate::types::{Tool, ToolResult};
use dhi_core::error::{DhiError, Result};
use std::fs;
use std::path::Path;

pub struct GetSnippetTool;

#[async_trait::async_trait]
impl Tool for GetSnippetTool {
    fn name(&self) -> &str {
        "get_snippet"
    }

    async fn execute(&self, args: serde_json::Value) -> Result<ToolResult> {
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

        let file_path = Path::new(path);
        if !file_path.is_file() {
            return Err(DhiError::ToolExecution(format!("{} is not a file", path)));
        }

        let content =
            fs::read_to_string(file_path).map_err(|e| DhiError::ToolExecution(e.to_string()))?;
        let lines: Vec<&str> = content.lines().collect();

        if start >= end || start >= lines.len() {
            return Err(DhiError::ToolExecution("Invalid line range".to_string()));
        }

        let snippet: Vec<&str> = lines[start..end.min(lines.len())].to_vec();
        let output = snippet.join("\n");

        Ok(ToolResult {
            success: true,
            output,
            token_cost: 0,
        })
    }
}
