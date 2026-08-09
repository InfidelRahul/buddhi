use crate::types::ToolResult;

pub struct ToolResultCompressor {
    max_tokens: usize,
}

impl ToolResultCompressor {
    pub fn new(max_tokens: usize) -> Self {
        Self { max_tokens }
    }

    pub fn compress(&self, result: &mut ToolResult) {
        // Simple heuristic: 1 token ≈ 4 characters
        let max_chars = self.max_tokens * 4;
        if result.output.len() > max_chars {
            result.output.truncate(max_chars);
            result.output.push_str("\n... [TRUNCATED]");
        }
        result.token_cost = result.output.len() / 4;
    }
}
