use dhi_core::error::{DhiError, Result};
use dhi_security::path_guard::PathGuard;
use serde_json::Value;
use std::fs;
use std::path::PathBuf;

pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: Value,
}

pub struct ToolExecutor {
    root: PathBuf,
}

impl ToolExecutor {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    pub fn execute(&self, tool_call: &ToolCall) -> Result<String> {
        match tool_call.name.as_str() {
            "read_file" => {
                let path = tool_call.arguments["path"]
                    .as_str()
                    .ok_or_else(|| DhiError::Config("Missing 'path' argument".to_string()))?;
                let safe_path = PathGuard::validate(path, &self.root)?;
                let content = fs::read_to_string(&safe_path).map_err(DhiError::Io)?;
                Ok(content)
            }
            "write_file" => {
                let path = tool_call.arguments["path"]
                    .as_str()
                    .ok_or_else(|| DhiError::Config("Missing 'path' argument".to_string()))?;
                let content = tool_call.arguments["content"]
                    .as_str()
                    .ok_or_else(|| DhiError::Config("Missing 'content' argument".to_string()))?;
                let safe_path = PathGuard::validate(path, &self.root)?;

                if let Some(parent) = safe_path.parent() {
                    fs::create_dir_all(parent).map_err(DhiError::Io)?;
                }
                fs::write(&safe_path, content).map_err(DhiError::Io)?;
                Ok("File written successfully.".to_string())
            }
            _ => Err(DhiError::Config(format!(
                "Unknown tool: {}",
                tool_call.name
            ))),
        }
    }
}
