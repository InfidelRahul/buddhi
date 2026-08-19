pub struct ErrorCompressor {
    max_chars: usize,
}

impl ErrorCompressor {
    pub fn new(max_tokens: usize) -> Self {
        // Heuristic: 1 token ≈ 4 characters
        Self {
            max_chars: max_tokens * 4,
        }
    }

    pub fn compress(&self, errors: &str) -> String {
        if errors.len() <= self.max_chars {
            return errors.to_string();
        }

        // Keep the beginning (context) and the end (final error summary)
        // This ensures the model sees both the origin and the final failure state
        let half = self.max_chars / 2;
        let start = &errors[..half];
        let end = &errors[errors.len() - half..];

        format!(
            "{}\n... [ERRORS TRUNCATED FOR TOKEN BUDGET] ...\n{}",
            start, end
        )
    }
}
