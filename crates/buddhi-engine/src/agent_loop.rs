use crate::context_manager::ContextManager;
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
    context_manager: Mutex<ContextManager>,
}

impl AgentLoop {
    pub fn new(project_root: PathBuf, api_key: String, model: String) -> Self {
        Self {
            project_root,
            max_retries: 3,
            llm_provider: Arc::new(Mutex::new(OpenAiProvider::new(api_key, model))),
            context_manager: Mutex::new(ContextManager::new()),
        }
    }

    pub async fn run(&self, task: &str) -> Result<()> {
        let executor = ToolExecutor::new(self.project_root.clone());
        let verifier = VerifyRunner::new(self.project_root.clone());

        // LOCAL INTELLIGENCE PHASE
        let mut ctx = self.context_manager.lock().await;
        let project_summary = ctx.analyze_project(&self.project_root);
        let rag_context = ctx.build_rag_context(task, &self.project_root);
        drop(ctx);

        tracing::info!("{}", project_summary);
        tracing::info!("RAG context length: {} chars", rag_context.len());

        let mut messages = vec![
            json!({
                "role": "system",
                "content": format!(
                    "You are Buddhi, an autonomous coding agent.\n{}\n\n{}\n\n\
                     Tools: read_file, write_file. Complete the task. \
                     Verify changes compile.",
                    project_summary, rag_context
                )
            }),
            json!({"role": "user", "content": task}),
        ];

        for attempt in 0..=self.max_retries {
            tracing::info!("Agent attempt {}/{}", attempt, self.max_retries);
            let response = self.call_llm(&messages).await?;

            match self.parse_response(&response) {
                LlmAction::ToolCall(call) => {
                    tracing::info!("Tool: {}", call.name);
                    let tool_result = executor
                        .execute(&call)
                        .unwrap_or_else(|e| format!("Tool failed: {}", e));

                    messages.push(json!({"role": "assistant", "content": &response}));
                    messages.push(json!({
                        "role": "tool",
                        "tool_call_id": call.id,
                        "content": tool_result
                    }));

                    if call.name == "write_file" {
                        let verification = verifier.run_cargo_check()?;
                        if verification.is_success() {
                            tracing::info!("Verification passed.");
                            return Ok(());
                        }
                        messages.push(json!({
                            "role": "user",
                            "content": format!("Compilation errors:\n{}\nFix these.",
                                verification.compress_errors())
                        }));
                    }
                }
                LlmAction::Text(text) => {
                    tracing::info!("Agent response: {}", text);
                    return Ok(());
                }
                LlmAction::Malformed(err) => {
                    messages.push(json!({
                        "role": "user",
                        "content": format!("Malformed response: {}. Retry with valid JSON.", err)
                    }));
                }
            }
        }

        Err(BuddhiError::Config("Max retries exceeded.".into()))
    }

    async fn call_llm(&self, messages: &[Value]) -> Result<String> {
        let provider = self.llm_provider.lock().await;
        let body = json!({
            "model": provider.model(),
            "messages": messages,
            "stream": false,
            "tools": [{
                "type": "function",
                "function": {
                    "name": "read_file",
                    "description": "Read file contents",
                    "parameters": {
                        "type": "object",
                        "properties": {"path": {"type": "string"}},
                        "required": ["path"]
                    }
                }
            }, {
                "type": "function",
                "function": {
                    "name": "write_file",
                    "description": "Write content to file",
                    "parameters": {
                        "type": "object",
                        "properties": {
                            "path": {"type": "string"},
                            "content": {"type": "string"}
                        },
                        "required": ["path", "content"]
                    }
                }
            }]
        });

        provider
            .complete(&body)
            .await
            .map_err(|e| BuddhiError::Config(format!("LLM call failed: {}", e)))
    }

    fn parse_response(&self, raw: &str) -> LlmAction {
        let Ok(val) = serde_json::from_str::<Value>(raw) else {
            return LlmAction::Text(raw.to_string());
        };

        if let Some(calls) = val["choices"][0]["message"]["tool_calls"].as_array() {
            if let Some(first) = calls.first() {
                let id = first["id"].as_str().unwrap_or("call_0").to_string();
                let name = first["function"]["name"]
                    .as_str()
                    .unwrap_or_default()
                    .to_string();
                let args_raw = first["function"]["arguments"].as_str().unwrap_or("{}");
                let arguments = serde_json::from_str(args_raw).unwrap_or(json!({}));
                return LlmAction::ToolCall(ToolCall {
                    id,
                    name,
                    arguments,
                });
            }
        }

        if let Some(content) = val["choices"][0]["message"]["content"].as_str() {
            return LlmAction::Text(content.to_string());
        }

        LlmAction::Malformed("Unrecognized response structure".into())
    }
}

enum LlmAction {
    ToolCall(ToolCall),
    Text(String),
    Malformed(String),
}
