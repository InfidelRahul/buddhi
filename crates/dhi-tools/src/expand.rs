use crate::types::{Tool, ToolResult};
use dhi_core::error::{DhiError, Result};
use std::fs;
use std::path::Path;

pub struct ExpandTool;

#[async_trait::async_trait]
impl Tool for ExpandTool {
    fn name(&self) -> &str {
        "expand"
    }

    async fn execute(&self, args: serde_json::Value) -> Result<ToolResult> {
        let path = args
            .get("path")
            .and_then(|p| p.as_str())
            .ok_or_else(|| DhiError::ToolExecution("Missing path argument".to_string()))?;

        let dir_path = Path::new(path);
        if !dir_path.is_dir() {
            return Err(DhiError::ToolExecution(format!(
                "{} is not a directory",
                path
            )));
        }

        let mut output = String::new();
        let entries = fs::read_dir(dir_path).map_err(|e| DhiError::ToolExecution(e.to_string()))?;

        for entry in entries {
            let entry = entry.map_err(|e| DhiError::ToolExecution(e.to_string()))?;
            let file_name = entry.file_name().to_string_lossy().to_string();
            let file_type = if entry.path().is_dir() { "dir" } else { "file" };
            output.push_str(&format!("{} {}\n", file_type, file_name));
        }

        Ok(ToolResult {
            success: true,
            output,
            token_cost: 0, // Will be calculated by the compressor
        })
    }
}
