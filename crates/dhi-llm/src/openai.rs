use crate::provider::{ChatMessage, LlmProvider, LlmResponse, TokenUsage};
use async_trait::async_trait;
use dhi_core::error::{DhiError, Result};
use futures_util::{Stream, StreamExt};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::pin::Pin;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct OpenAiRequest {
    model: String,
    messages: Vec<ChatMessage>,
    temperature: f32,
    stream: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct OpenAiResponse {
    choices: Vec<OpenAiChoice>,
    usage: Option<OpenAiUsage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct OpenAiChoice {
    message: ChatMessage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct OpenAiUsage {
    prompt_tokens: usize,
    completion_tokens: usize,
    total_tokens: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct OpenAiStreamChunk {
    choices: Vec<OpenAiStreamChoice>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct OpenAiStreamChoice {
    delta: OpenAiDelta,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct OpenAiDelta {
    content: Option<String>,
}

pub struct OpenAiClient {
    client: Client,
    api_key: String,
    base_url: String,
    model: String,
}

impl OpenAiClient {
    pub fn new(api_key: String, base_url: String, model: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            base_url,
            model,
        }
    }
}

#[async_trait]
impl LlmProvider for OpenAiClient {
    async fn chat(&self, messages: Vec<ChatMessage>) -> Result<LlmResponse> {
        let request_body = OpenAiRequest {
            model: self.model.clone(),
            messages,
            temperature: 0.1,
            stream: false,
        };

        let response = self
            .client
            .post(format!("{}/chat/completions", self.base_url))
            .bearer_auth(&self.api_key)
            .json(&request_body)
            .send()
            .await
            .map_err(|e| DhiError::Config(format!("OpenAI request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(DhiError::Config(format!(
                "OpenAI API error ({}): {}",
                status, error_text
            )));
        }

        let openai_response: OpenAiResponse = response
            .json()
            .await
            .map_err(|e| DhiError::Config(format!("Failed to parse OpenAI response: {}", e)))?;

        let content = openai_response
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .unwrap_or_default();

        let usage = openai_response.usage.map(|u| TokenUsage {
            prompt_tokens: u.prompt_tokens,
            completion_tokens: u.completion_tokens,
            total_tokens: u.total_tokens,
        });

        Ok(LlmResponse { content, usage })
    }

    async fn chat_stream(
        &self,
        messages: Vec<ChatMessage>,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<String>> + Send>>> {
        let request_body = OpenAiRequest {
            model: self.model.clone(),
            messages,
            temperature: 0.1,
            stream: true,
        };

        let response = self
            .client
            .post(format!("{}/chat/completions", self.base_url))
            .bearer_auth(&self.api_key)
            .json(&request_body)
            .send()
            .await
            .map_err(|e| DhiError::Config(format!("OpenAI stream request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(DhiError::Config(format!(
                "OpenAI API stream error ({}): {}",
                status, error_text
            )));
        }

        let byte_stream = response.bytes_stream();

        // Map the byte stream to a stream of parsed string chunks
        let mapped_stream = byte_stream.map(|chunk_result| {
            let chunk =
                chunk_result.map_err(|e| DhiError::Config(format!("Stream read error: {}", e)))?;
            let text = String::from_utf8_lossy(&chunk).to_string();

            // OpenAI SSE format sends lines like "data: {...}"
            let mut extracted_content = String::new();
            for line in text.lines() {
                if let Some(json_str) = line.strip_prefix("data: ") {
                    if json_str.trim() == "[DONE]" {
                        continue;
                    }
                    if let Ok(parsed) = serde_json::from_str::<OpenAiStreamChunk>(json_str) {
                        if let Some(content) =
                            parsed.choices.first().and_then(|c| c.delta.content.clone())
                        {
                            extracted_content.push_str(&content);
                        }
                    }
                }
            }
            Ok(extracted_content)
        });

        Ok(Box::pin(mapped_stream))
    }
}
