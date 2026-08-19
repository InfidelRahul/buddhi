use buddhi_core::error::{BuddhiError, Result};
use buddhi_llm::openai::OpenAiProvider;
use buddhi_tools::executor::{ToolCall, ToolExecutor};
use buddhi_verify::runner::VerifyRunner;
use serde_json::{json, Value};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct AgentLoop {
    project_root: PathBuf,
    max_retries: usize,
    llm_provider: Arc<Mutex<OpenAiProvider>>,
}

impl AgentLoop {
    pub fn new(project_root: PathBuf, api_key: String, model: String) -> Self {
        let provider = OpenAiProvider::new(api_key, model);
        Self {
            project_root,
            max_retries: 3,
            llm_provider: Arc::new(Mutex::new(provider)),
        }
    }

    pub async fn run(&self, task: &str) -> Result<()> {
        let executor = ToolExecutor::new(self.project_root.clone());
        let verifier = VerifyRunner::new(self.project_root.clone());

        let mut context = vec![
            json!({
                "role": "system",
                "content": "You are Buddhi, an autonomous coding agent. You have access to tools: read_file, write_file. Use them to complete the task. Always verify your changes compile successfully."
            }),
            json!({"role": "user", "content": task}),
        ];

        for attempt in 0..=self.max_retries {
            tracing::info!("Agent loop attempt {} of {}", attempt, self.max_retries);

            // 1. Call the real Cloud LLM with streaming
            let response = self.call_cloud_llm(&context).await?;

            // 2. Parse the response for tool calls or final text
            match self.parse_llm_response(&response) {
                LlmResponse::ToolCall(call) => {
                    tracing::info!("Executing tool: {}", call.name);
                    let result_str = match executor.execute(&call) {
                        Ok(out) => out,
                        Err(e) => format!("Tool execution failed: {}", e),
                    };

                    context.push(json!({"role": "assistant", "content": response.clone()}));
                    context.push(json!({
                        "role": "tool",
                        "tool_call_id": call.id,
                        "content": result_str
                    }));

                    // 3. Verify if tool modified the filesystem
                    if call.name == "write_file" {
                        let verification = verifier.run_cargo_check()?;
                        if verification.is_success() {
                            tracing::info!("Verification passed! Task complete.");
                            return Ok(());
                        } else {
                            tracing::warn!("Verification failed. Feeding errors back to LLM...");
                            let error_msg = format!(
                                "The code you wrote caused the following compilation errors:\n{}\nPlease fix these errors.",
                                verification.compress_errors()
                            );
                            context.push(json!({"role": "user", "content": error_msg}));
                            continue;
                        }
                    }
                }
                LlmResponse::Text(text) => {
                    tracing::info!("Agent finished with text response.");
                    tracing::info!("Response: {}", text);
                    return Ok(());
                }
                LlmResponse::Error(e) => {
                    tracing::error!("LLM response parsing failed: {}", e);
                    context.push(json!({
                        "role": "user",
                        "content": format!("Your previous response was malformed: {}. Please try again with valid JSON.", e)
                    }));
                    continue;
                }
            }
        }

        Err(BuddhiError::Config(
            "Max retries exceeded. Task failed.".to_string(),
        ))
    }

    async fn call_cloud_llm(&self, context: &[Value]) -> Result<String> {
        let provider = self.llm_provider.lock().await;
        let request_body = json!({
            "model": provider.model(),
            "messages": context,
            "stream": false,
            "tools": [
                {
                    "type": "function",
                    "function": {
                        "name": "read_file",
                        "description": "Read the contents of a file",
                        "parameters": {
                            "type": "object",
                            "properties": {
                                "path": {"type": "string", "description": "The file path to read"}
                            },
                            "required": ["path"]
                        }
                    }
                },
                {
                    "type": "function",
                    "function": {
                        "name": "write_file",
                        "description": "Write content to a file",
                        "parameters": {
                            "type": "object",
                            "properties": {
                                "path": {"type": "string", "description": "The file path to write to"},
                                "content": {"type": "string", "description": "The content to write"}
                            },
                            "required": ["path", "content"]
                        }
                    }
                }
            ]
        });

        provider
            .complete(&request_body)
            .await
            .map_err(|e| BuddhiError::Config(format!("Cloud LLM call failed: {}", e)))
    }

    fn parse_llm_response(&self, response: &str) -> LlmResponse {
        // Try to parse as JSON first (tool call)
        if let Ok(json_val) = serde_json::from_str::<Value>(response) {
            if let Some(tool_calls) = json_val["choices"][0]["message"]["tool_calls"].as_array() {
                if let Some(first_call) = tool_calls.first() {
                    let id = first_call["id"].as_str().unwrap_or("call_1").to_string();
                    let name = first_call["function"]["name"]
                        .as_str()
                        .unwrap_or("")
                        .to_string();
                    let args_str = first_call["function"]["arguments"].as_str().unwrap_or("{}");
                    let arguments: Value = serde_json::from_str(args_str).unwrap_or(json!({}));

                    return LlmResponse::ToolCall(ToolCall {
                        id,
                        name,
                        arguments,
                    });
                }
            }
            // Try to extract text content
            if let Some(content) = json_val["choices"][0]["message"]["content"].as_str() {
                return LlmResponse::Text(content.to_string());
            }
        }

        // If it's not valid JSON, treat it as plain text
        LlmResponse::Text(response.to_string())
    }
}

enum LlmResponse {
    ToolCall(ToolCall),
    Text(String),
    Error(String),
}
