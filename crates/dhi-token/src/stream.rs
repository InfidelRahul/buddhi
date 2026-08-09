use crate::budget::TokenBudget;
use dhi_core::error::{DhiError, Result};

pub struct TokenStreamTracker {
    budget: TokenBudget,
    accumulated_text: String,
}

impl TokenStreamTracker {
    pub fn new(budget: TokenBudget) -> Self {
        Self {
            budget,
            accumulated_text: String::new(),
        }
    }

    pub fn process_chunk(&mut self, chunk: &str) -> Result<()> {
        self.accumulated_text.push_str(chunk);

        // Count tokens for the accumulated text to ensure accuracy across chunk boundaries.
        // Note: This re-counts the entire stream on every chunk.
        // Optimization: Implement incremental token counting in Phase 3 if performance becomes critical.
        let total_tokens = self.budget.count_text(&self.accumulated_text)?;

        if total_tokens > self.budget.max_tokens() {
            return Err(DhiError::BudgetExceeded(format!(
                "Stream exceeded budget: {} tokens",
                total_tokens
            )));
        }

        Ok(())
    }

    pub fn get_accumulated_text(&self) -> &str {
        &self.accumulated_text
    }
}
