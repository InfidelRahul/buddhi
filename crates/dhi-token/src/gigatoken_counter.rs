use crate::counter::TokenCounter;
use dhi_core::error::{DhiError, Result};
// Import the native Rust API from Gigatoken
// Note: Adjust the import path based on the actual Gigatoken Rust API structure
use gigatoken::Tokenizer; 

pub struct GigatokenCounter {
    tokenizer: Tokenizer,
}

impl GigatokenCounter {
    pub fn try_new(model_path: &str) -> Result<Self> {
        // Initialize the tokenizer with the model path or configuration
        // This is a placeholder for the actual Gigatoken initialization API
        let tokenizer = Tokenizer::from_pretrained(model_path)
            .map_err(|e| DhiError::Config(format!("Failed to load Gigatoken: {}", e)))?;
            
        Ok(Self { tokenizer })
    }
}

impl TokenCounter for GigatokenCounter {
    fn count_tokens(&self, text: &str) -> Result<usize> {
        // Use the native Rust encode method to count tokens
        // This avoids any Python overhead
        let tokens = self.tokenizer.encode(text, false)
            .map_err(|e| DhiError::Config(format!("Tokenization failed: {}", e)))?;
            
        Ok(tokens.len())
    }
}
