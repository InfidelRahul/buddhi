use crate::error::{DhiError, Result};

pub struct ContextManager {
    max_tokens: usize,
    history: Vec<String>,
}
impl ContextManager {
    pub fn new(max_tokens: usize) -> Self {
        Self {
            max_tokens,
            history: Vec::new(),
        }
    }
    pub fn add_message(&mut self, message: &str) -> Result<()> {
        let estimated_tokens = message.len() / 4;
        if estimated_tokens > self.max_tokens {
            return Err(DhiError::Config(format!(
                "Message exceeds context window: {} tokens",
                estimated_tokens
            )));
        }
        self.history.push(message.to_string());
        self.truncate_if_needed();
        Ok(())
    }
    pub fn get_context(&self) -> String {
        self.history.join("\n")
    }
    fn truncate_if_needed(&mut self) {
        while self.history.iter().map(|m| m.len() / 4).sum::<usize>() > self.max_tokens
            && !self.history.is_empty()
        {
            self.history.remove(0);
        }
    }
}
