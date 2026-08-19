use crate::patch_safety::{PatchProposal, PatchSafety};
use crate::types::{Tool, ToolResult};
use buddhi_core::error::{DhiError, Result};
use buddhi_security::path_guard::PathGuard;
use std::path::Path;

pub struct ReplaceTool;

#[async_trait::async_trait]
impl Tool for ReplaceTool {
    fn name(&self) -> &str {
        "replace"
    }

    async fn execute(&self, args: serde_json::Value, project_root: &Path) -> Result<ToolResult> {
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

        let dry_run = args
            .get("dry_run")
            .and_then(|d| d.as_bool())
            .unwrap_or(false);

        let safe_path = PathGuard::validate(path, project_root)?;

        let proposal = PatchProposal {
            path: &safe_path,
            original,
            replacement,
            dry_run,
        };

        let result = PatchSafety::apply(&proposal)?;

        let status = if result.applied {
            "Patch applied successfully"
        } else {
            "Dry-run completed. No changes written to disk."
        };

        Ok(ToolResult {
            success: true,
            output: format!("{}\n\n{}", status, result.diff),
            token_cost: 0,
        })
    }
}
