use dhi_core::error::{DhiError, Result};
use std::path::Path;
use tokenizers::Tokenizer;

pub struct LocalTokenizer {
    tokenizer: Tokenizer,
}

impl LocalTokenizer {
    pub fn load(path: &Path) -> Result<Self> {
        let tokenizer = Tokenizer::from_file(path)
            .map_err(|e| DhiError::Config(format!("Failed to load tokenizer: {}", e)))?;
        Ok(Self { tokenizer })
    }

    pub fn encode(&self, text: &str) -> Result<Vec<u32>> {
        let encoding = self
            .tokenizer
            .encode(text, false)
            .map_err(|e| DhiError::Config(format!("Tokenization failed: {}", e)))?;
        Ok(encoding.get_ids().to_vec())
    }

    pub fn decode(&self, tokens: &[u32]) -> Result<String> {
        self.tokenizer
            .decode(tokens, true)
            .map_err(|e| DhiError::Config(format!("Decoding failed: {}", e)))
    }
}
