use buddhi_core::error::{BuddhiError, Result};
use std::path::{Path, PathBuf};

pub struct PathGuard;

impl PathGuard {
    pub fn validate(requested_path: &str, project_root: &Path) -> Result<PathBuf> {
        let canonical_root = project_root
            .canonicalize()
            .map_err(|e| BuddhiError::Config(format!("Failed to canonicalize root: {}", e)))?;

        let full_path = if Path::new(requested_path).is_absolute() {
            PathBuf::from(requested_path)
        } else {
            canonical_root.join(requested_path)
        };

        let canonical_path = full_path.canonicalize().map_err(|e| {
            BuddhiError::ToolExecution(format!("Path does not exist or cannot be resolved: {}", e))
        })?;

        if !canonical_path.starts_with(&canonical_root) {
            return Err(BuddhiError::ToolExecution(
                "Path traversal detected: outside project root".to_string(),
            ));
        }

        if let Some(file_name) = canonical_path.file_name().and_then(|n| n.to_str()) {
            if file_name.starts_with('.') || file_name == ".env" || file_name == "Cargo.lock" {
                return Err(BuddhiError::ToolExecution(format!(
                    "Access denied to hidden/sensitive file: {}",
                    file_name
                )));
            }
        }

        Ok(canonical_path)
    }
}
