use buddhi_core::error::{DhiError, Result};
use buddhi_tools::executor::{ToolCall, ToolExecutor};
use buddhi_verify::runner::VerifyRunner;
use serde_json::json;
use std::path::PathBuf;

pub struct AgentLoop {
    project_root: PathBuf,
    max_retries: usize,
}
impl AgentLoop {
    pub fn new(project_root: PathBuf) -> Self {
        Self {
            project_root,
            max_retries: 3,
        }
    }

    pub async fn run(&self, task: &str) -> Result<()> {
        let executor = ToolExecutor::new(self.project_root.clone());
        let verifier = VerifyRunner::new(self.project_root.clone());
        let mut context = vec![
            json!({"role": "system", "content": "You are an autonomous coding agent. Use tools to write code."}),
            json!({"role": "user", "content": task}),
        ];

        for attempt in 0..=self.max_retries {
            tracing::info!("Agent loop attempt {}", attempt);
            let tool_call = self.mock_llm_call(attempt, task);
            if let Some(call) = tool_call {
                tracing::info!("Executing tool: {}", call.name);
                let result_str = match executor.execute(&call) {
                    Ok(out) => out,
                    Err(e) => format!("Tool execution failed: {}", e),
                };
                context
                    .push(json!({"role": "tool", "tool_call_id": call.id, "content": result_str}));
                if call.name == "write_file" {
                    let verification = verifier.run_cargo_check()?;
                    if verification.is_success() {
                        tracing::info!("Verification passed! Task complete.");
                        return Ok(());
                    } else {
                        tracing::warn!("Verification failed. Feeding errors back to LLM...");
                        let error_msg = format!(
                            "The code you wrote caused the following compilation errors:\n{}",
                            verification.compress_errors()
                        );
                        context.push(json!({"role": "user", "content": error_msg}));
                        continue;
                    }
                }
            } else {
                tracing::info!("Agent finished with text response.");
                return Ok(());
            }
        }
        Err(DhiError::Config(
            "Max retries exceeded. Task failed.".to_string(),
        ))
    }

    fn mock_llm_call(&self, attempt: usize, _task: &str) -> Option<ToolCall> {
        if attempt == 0 {
            Some(ToolCall {
                id: "call_1".to_string(),
                name: "write_file".to_string(),
                arguments: json!({"path": "src/test_agent.rs", "content": "pub fn broken_code() { let x: i32 = \"string\"; }"}),
            })
        } else if attempt == 1 {
            Some(ToolCall {
                id: "call_2".to_string(),
                name: "write_file".to_string(),
                arguments: json!({"path": "src/test_agent.rs", "content": "pub fn working_code() { let x: i32 = 42; }"}),
            })
        } else {
            None
        }
    }
}
