use buddhi_core::error::{BuddhiError, Result};
use std::path::Path;
use tokenizers::Tokenizer;

/// Wraps the HuggingFace tokenizers crate for BPE/SentencePiece encoding.
pub struct BuddhiTokenizer {
    inner: Tokenizer,
}

impl BuddhiTokenizer {
    /// Load tokenizer from a JSON file.
    pub fn from_file(path: &Path) -> Result<Self> {
        let inner = Tokenizer::from_file(path)
            .map_err(|e| BuddhiError::Config(format!("Failed to load tokenizer: {}", e)))?;
        Ok(Self { inner })
    }

    /// Encode text into token IDs. Zero-copy where possible.
    pub fn encode(&self, text: &str) -> Result<Vec<u32>> {
        let encoding = self
            .inner
            .encode(text, false)
            .map_err(|e| BuddhiError::Config(format!("Encoding failed: {}", e)))?;
        Ok(encoding.get_ids().to_vec())
    }

    /// Decode token IDs back to text.
    pub fn decode(&self, ids: &[u32]) -> Result<String> {
        self.inner
            .decode(ids, true)
            .map_err(|e| BuddhiError::Config(format!("Decoding failed: {}", e)))
    }

    pub fn vocab_size(&self) -> usize {
        self.inner.get_vocab_size(true)
    }
}
