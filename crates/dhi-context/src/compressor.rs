pub struct ContextCompressor {
    max_chars: usize,
}

impl ContextCompressor {
    pub fn new(max_tokens: usize) -> Self {
        // Heuristic: 1 token ≈ 4 characters
        Self {
            max_chars: max_tokens * 4,
        }
    }

    pub fn compress(&self, context: &str) -> String {
        if context.len() <= self.max_chars {
            return context.to_string();
        }

        let truncated = &context[..self.max_chars];
        format!("{}...\n[CONTEXT TRUNCATED TO FIT TOKEN BUDGET]", truncated)
    }
}
