use crate::counter::TokenCounter;
use buddhi_core::error::{BuddhiError, Result};
use std::sync::Arc;

pub struct TokenBudget {
    counter: Arc<dyn TokenCounter>,
    max_tokens_per_turn: usize,
    used_tokens: usize,
}

impl TokenBudget {
    pub fn new(counter: Arc<dyn TokenCounter>, max_tokens_per_turn: usize) -> Self {
        Self {
            counter,
            max_tokens_per_turn,
            used_tokens: 0,
        }
    }

    pub fn check_and_add(&mut self, text: &str) -> Result<()> {
        let tokens = self.counter.count_tokens(text)?;
        if self.used_tokens + tokens > self.max_tokens_per_turn {
            return Err(BuddhiError::BudgetExceeded(format!(
                "Attempted to add {} tokens, but only {} remain",
                tokens,
                self.max_tokens_per_turn.saturating_sub(self.used_tokens)
            )));
        }
        self.used_tokens += tokens;
        Ok(())
    }

    pub fn count_text(&self, text: &str) -> Result<usize> {
        self.counter.count_tokens(text)
    }

    pub fn max_tokens(&self) -> usize {
        self.max_tokens_per_turn
    }

    pub fn remaining(&self) -> usize {
        self.max_tokens_per_turn.saturating_sub(self.used_tokens)
    }

    pub fn reset(&mut self) {
        self.used_tokens = 0;
    }
}
