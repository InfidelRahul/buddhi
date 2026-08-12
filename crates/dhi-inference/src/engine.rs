use dhi_core::error::Result;

pub trait InferenceEngine: Send + Sync {
    fn generate(&mut self, prompt: &str, max_tokens: usize) -> Result<String>;
    fn engine_type(&self) -> &'static str;
}
