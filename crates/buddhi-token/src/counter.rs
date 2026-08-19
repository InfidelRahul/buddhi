use buddhi_core::error::Result;

pub trait TokenCounter: Send + Sync {
    fn count_tokens(&self, text: &str) -> Result<usize>;
}

pub struct CharCounter;

impl TokenCounter for CharCounter {
    fn count_tokens(&self, text: &str) -> Result<usize> {
        // Placeholder: Replace with Gigatoken integration in Phase 2 Part 2
        Ok(text.chars().count())
    }
}
