use async_trait::async_trait;
use buddhi_core::error::Result;
use futures_util::Stream;
use serde::{Deserialize, Serialize};
use std::pin::Pin;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmResponse {
    pub content: String,
    pub usage: Option<TokenUsage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsage {
    pub prompt_tokens: usize,
    pub completion_tokens: usize,
    pub total_tokens: usize,
}

#[async_trait]
pub trait LlmProvider: Send + Sync {
    async fn chat(&self, messages: Vec<ChatMessage>) -> Result<LlmResponse>;

    // Returns a stream of string chunks
    async fn chat_stream(
        &self,
        messages: Vec<ChatMessage>,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<String>> + Send>>>;
}
