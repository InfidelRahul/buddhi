use buddhi_context::extractor::BuddhiExtractor;
use buddhi_context::scanner::ProjectScanner;
use std::path::Path;

/// The bridge between the Local Scout and the Cloud Architect.
/// It prepares the project context locally before the LLM loop starts.
pub struct ContextManager {
    extractor: BuddhiExtractor,
}

impl ContextManager {
    pub fn new() -> Self {
        Self {
            extractor: BuddhiExtractor::new(),
        }
    }

    /// Scans the project and builds a structural summary for the LLM.
    pub fn analyze_project(&self, root: &Path) -> String {
        let languages = ProjectScanner::scan(root);
        if languages.is_empty() {
            "No specific project languages detected.".to_string()
        } else {
            format!("Detected project language stack: {:?}", languages)
        }
    }

    /// Surgically extracts a code chunk from a specific file.
    pub fn extract_code_chunk(
        &mut self,
        root: &Path,
        file_path: &str,
        language: &str,
        search_term: &str,
    ) -> Option<String> {
        let full_path = root.join(file_path);
        match std::fs::read_to_string(&full_path) {
            Ok(code) => self.extractor.extract_context(&code, language, search_term),
            Err(_) => None,
        }
    }
}

impl Default for ContextManager {
    fn default() -> Self {
        Self::new()
    }
}
