use buddhi_core::error::Result;
pub trait InferenceEngine: Send + Sync {
    fn generate(&mut self, prompt: &str, max_tokens: usize) -> Result<String>;
    fn generate_stream(
        &mut self,
        prompt: &str,
        max_tokens: usize,
        on_token: &mut (dyn FnMut(&str) + Send),
    ) -> Result<String>;
    fn engine_type(&self) -> &'static str;
}
