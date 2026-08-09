use dhi_core::error::{DhiError, Result};
use dhi_core::types::ToolCall;
use serde_json::Value;

pub struct StreamInterceptor {
    buffer: String,
}

impl StreamInterceptor {
    pub fn new() -> Self {
        Self {
            buffer: String::new(),
        }
    }

    pub fn process_chunk(&mut self, chunk: &str) -> Result<Option<ToolCall>> {
        self.buffer.push_str(chunk);

        // Check for prose before the first '{'
        if let Some(start) = self.buffer.find('{') {
            let prefix = self.buffer[..start].trim();
            if !prefix.is_empty() {
                return Err(DhiError::Config(format!("Prose detected: {}", prefix)));
            }
        } else {
            // No '{' yet, check if the buffer is just whitespace or prose
            if !self.buffer.trim().is_empty() {
                return Err(DhiError::Config(format!(
                    "Prose detected: {}",
                    self.buffer.trim()
                )));
            }
            return Ok(None);
        }

        // Try to parse the buffer from the first '{' to the end
        if let Some(start) = self.buffer.find('{') {
            let json_str = &self.buffer[start..];
            match serde_json::from_str::<Value>(json_str) {
                Ok(json) => {
                    self.buffer.clear();
                    // Expected format: {"tool": "name", "args": {...}}
                    if let (Some(name), Some(args)) = (json.get("tool"), json.get("args")) {
                        if let (Some(name_str), Some(args_val)) = (name.as_str(), Some(args)) {
                            return Ok(Some(ToolCall {
                                name: name_str.to_string(),
                                arguments: args_val.clone(),
                            }));
                        }
                    }
                    return Err(DhiError::Config("Invalid tool call schema".to_string()));
                }
                Err(e) => {
                    // If it's an EOF error, it means the JSON is incomplete
                    if e.is_eof() {
                        return Ok(None);
                    }
                    return Err(DhiError::Config(format!("JSON parse error: {}", e)));
                }
            }
        }

        Ok(None)
    }
}
