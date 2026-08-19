use crate::counter::TokenCounter;
use buddhi_core::error::{DhiError, Result};
use gigatoken_rs::load_tokenizer::hf::{load_hf_slice, HfTokenizer};
use std::fs;
use std::sync::Mutex;

pub struct GigatokenCounter {
    // Mutex is required because gigatoken's BPE memoized_encode requires &mut self
    // to update its high-speed internal cache during tokenization.
    tokenizer: Mutex<HfTokenizer>,
}

impl GigatokenCounter {
    pub fn try_new(tokenizer_json_path: &str) -> Result<Self> {
        let data = fs::read(tokenizer_json_path)
            .map_err(|e| DhiError::Config(format!("Failed to read tokenizer file: {}", e)))?;

        let tokenizer = load_hf_slice(&data)
            .map_err(|e| DhiError::Config(format!("Failed to load Gigatoken: {}", e)))?;

        Ok(Self {
            tokenizer: Mutex::new(tokenizer),
        })
    }
}

impl TokenCounter for GigatokenCounter {
    fn count_tokens(&self, text: &str) -> Result<usize> {
        let mut tok = self
            .tokenizer
            .lock()
            .map_err(|e| DhiError::Config(format!("Failed to acquire tokenizer lock: {}", e)))?;

        match &mut *tok {
            HfTokenizer::Bpe(bpe_tok) => {
                // Gigatoken requires pretokenization before BPE encoding
                let pretokens = gigatoken_rs::pretokenize::pretokenize_as_iter(text.as_bytes());
                let mut count = 0;
                bpe_tok.memoized_encode(pretokens, |tokens| {
                    count += tokens.len();
                });
                Ok(count)
            }
            HfTokenizer::SentencePiece(sp_tok) => {
                let ids = sp_tok.encode_raw(text);
                Ok(ids.len())
            }
        }
    }
}
