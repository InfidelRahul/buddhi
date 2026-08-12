use dhi_core::error::Result;

pub trait InferenceEngine: Send + Sync {
    fn generate(&mut self, prompt: &str, max_tokens: usize) -> Result<String>;
    fn generate_stream<F>(
        &mut self,
        prompt: &str,
        max_tokens: usize,
        on_token: F,
    ) -> Result<String>
    where
        F: FnMut(&str);
    fn engine_type(&self) -> &'static str;
}
