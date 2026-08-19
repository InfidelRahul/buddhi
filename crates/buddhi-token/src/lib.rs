use buddhi_core::error::{BuddhiError, Result};
use rs_gigatoken::load_tokenizer::hf::load_hf_bpe;
use rs_gigatoken::Tokenizer;
use std::path::PathBuf;
use std::sync::Mutex;

pub trait TokenCounter: Send + Sync {
    fn count(&self, text: &str) -> usize;
}

/// Fast character-based fallback (1 token ≈ 4 chars)
pub struct FastEstimator;
impl TokenCounter for FastEstimator {
    fn count(&self, text: &str) -> usize {
        let chars = text.chars().count();
        std::cmp::max(1, (chars + 3) / 4)
    }
}

/// High-throughput rs_gigatoken counter
pub struct GigatokenCounter {
    tokenizer: Mutex<Tokenizer>,
}

impl GigatokenCounter {
    pub fn try_new(tokenizer_path: &PathBuf) -> Result<Self> {
        let path_str = tokenizer_path.to_str().unwrap_or("tokenizer.json");
        let tokenizer = load_hf_bpe(path_str)
            .map_err(|e| BuddhiError::Config(format!("Failed to load gigatoken: {}", e)))?;
        Ok(Self {
            tokenizer: Mutex::new(tokenizer),
        })
    }
}

impl TokenCounter for GigatokenCounter {
    fn count(&self, text: &str) -> usize {
        if let Ok(mut guard) = self.tokenizer.lock() {
            let mut tokens = Vec::new();
            guard.memoized_encode(
                rs_gigatoken::pretokenize::pretokenize_as_iter(text.as_bytes()),
                |ids| tokens.extend_from_slice(ids),
            );
            tokens.len()
        } else {
            // Fallback on lock poisoning
            text.chars().count() / 4
        }
    }
}

pub struct TokenBudget {
    pub max_tokens_per_turn: usize,
    pub max_total_tokens: usize,
    pub used_total: usize,
    counter: Box<dyn TokenCounter>,
}

impl TokenBudget {
    pub fn new(max_per_turn: usize, max_total: usize, tokenizer_path: Option<PathBuf>) -> Self {
        let counter: Box<dyn TokenCounter> = if let Some(path) = tokenizer_path {
            match GigatokenCounter::try_new(&path) {
                Ok(gc) => {
                    tracing::info!("Loaded rs-gigatoken successfully from {}", path.display());
                    Box::new(gc)
                }
                Err(e) => {
                    tracing::warn!(
                        "Failed to load rs-gigatoken, falling back to FastEstimator: {}",
                        e
                    );
                    Box::new(FastEstimator)
                }
            }
        } else {
            Box::new(FastEstimator)
        };

        Self {
            max_tokens_per_turn: max_per_turn,
            max_total_tokens: max_total,
            used_total: 0,
            counter,
        }
    }

    pub fn check_turn(&self, text: &str) -> Result<bool> {
        let tokens = self.counter.count(text);
        if tokens > self.max_tokens_per_turn {
            return Err(BuddhiError::Config(format!(
                "Turn budget exceeded: {} tokens (limit: {})",
                tokens, self.max_tokens_per_turn
            )));
        }
        Ok(true)
    }

    pub fn record_usage(&mut self, text: &str) {
        let tokens = self.counter.count(text);
        self.used_total += tokens;
    }

    pub fn check_total(&self) -> bool {
        self.used_total <= self.max_total_tokens
    }
}
