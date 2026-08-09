use dhi_core::error::Result;
use dhi_core::types::TaskContract;
use dhi_llm::parser::StreamInterceptor;
use dhi_llm::provider::{ChatMessage, LlmProvider};
use dhi_token::budget::TokenBudget;
use futures_util::StreamExt;

pub struct AgentLoop<'a> {
    provider: &'a dyn LlmProvider,
    budget: TokenBudget,
}

impl<'a> AgentLoop<'a> {
    pub fn new(provider: &'a dyn LlmProvider, budget: TokenBudget) -> Self {
        Self { provider, budget }
    }

    pub async fn run(&mut self, _contract: &TaskContract, system_prompt: &str) -> Result<()> {
        let messages = vec![
            ChatMessage {
                role: "system".to_string(),
                content: system_prompt.to_string(),
            },
            ChatMessage {
                role: "user".to_string(),
                content: "Execute task.".to_string(),
            },
        ];

        let mut stream = self.provider.chat_stream(messages).await?;
        let mut interceptor = StreamInterceptor::new();

        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result?;
            self.budget.check_and_add(&chunk)?;

            if let Some(tool_call) = interceptor.process_chunk(&chunk)? {
                tracing::info!("Tool call intercepted: {}", tool_call.name);
                // Tool execution, patch application, and verification
                // will be wired here in Phase 12.
                break; // Stop stream immediately after first valid tool call
            }
        }
        Ok(())
    }
}
