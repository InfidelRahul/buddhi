use crate::types::{Tool, ToolResult};
use dhi_core::error::{DhiError, Result};
use std::fs;
use std::path::Path;

pub struct ReplaceTool;

#[async_trait::async_trait]
impl Tool for ReplaceTool {
    fn name(&self) -> &str {
        "replace"
    }

    async fn execute(&self, args: serde_json::Value) -> Result<ToolResult> {
        let path = args
            .get("path")
            .and_then(|p| p.as_str())
            .ok_or_else(|| DhiError::ToolExecution("Missing path argument".to_string()))?;
        let original = args
            .get("original")
            .and_then(|o| o.as_str())
            .ok_or_else(|| DhiError::ToolExecution("Missing original argument".to_string()))?;
        let replacement = args
            .get("replacement")
            .and_then(|r| r.as_str())
            .ok_or_else(|| DhiError::ToolExecution("Missing replacement argument".to_string()))?;

        let file_path = Path::new(path);
        if !file_path.is_file() {
            return Err(DhiError::ToolExecution(format!("{} is not a file", path)));
        }

        let content =
            fs::read_to_string(file_path).map_err(|e| DhiError::ToolExecution(e.to_string()))?;

        if !content.contains(original) {
            return Err(DhiError::ToolExecution(
                "Original code not found in file".to_string(),
            ));
        }

        let new_content = content.replace(original, replacement);
        fs::write(file_path, new_content).map_err(|e| DhiError::ToolExecution(e.to_string()))?;

        Ok(ToolResult {
            success: true,
            output: "Replacement successful".to_string(),
            token_cost: 0,
        })
    }
}
