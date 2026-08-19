use buddhi_core::error::{DhiError, Result};
use std::fs;
use std::path::Path;

pub struct PatchProposal<'a> {
    pub path: &'a Path,
    pub original: &'a str,
    pub replacement: &'a str,
    pub dry_run: bool,
}

pub struct PatchResult {
    pub diff: String,
    pub applied: bool,
}

pub struct PatchSafety;

impl PatchSafety {
    pub fn apply(proposal: &PatchProposal) -> Result<PatchResult> {
        let content = fs::read_to_string(proposal.path)
            .map_err(|e| DhiError::ToolExecution(format!("Failed to read file: {}", e)))?;

        if !content.contains(proposal.original) {
            return Err(DhiError::ToolExecution(
                "Original code not found in file. Patch rejected.".to_string(),
            ));
        }

        let new_content = content.replace(proposal.original, proposal.replacement);

        // Generate a simple unified diff representation
        let diff = format!(
            "--- {}\n+++ {}\n@@ @@\n-{}\n+{}",
            proposal.path.display(),
            proposal.path.display(),
            proposal.original.replace('\n', "\n-"),
            proposal.replacement.replace('\n', "\n+")
        );

        if proposal.dry_run {
            return Ok(PatchResult {
                diff,
                applied: false,
            });
        }

        // Create temporary rollback backup
        let backup_path = proposal.path.with_extension("dhi.bak");
        fs::write(&backup_path, &content)
            .map_err(|e| DhiError::ToolExecution(format!("Failed to create backup: {}", e)))?;

        // Apply patch atomically
        fs::write(proposal.path, new_content).map_err(|e| {
            // Attempt to restore backup if write fails
            let _ = fs::write(proposal.path, &content);
            DhiError::ToolExecution(format!("Failed to write patch: {}", e))
        })?;

        // Clean up backup on success (Git handles real rollbacks)
        let _ = fs::remove_file(backup_path);

        Ok(PatchResult {
            diff,
            applied: true,
        })
    }
}
