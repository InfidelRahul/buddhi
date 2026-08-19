use buddhi_context::extractor::BuddhiExtractor;
use buddhi_context::scanner::ProjectScanner;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::Path;

#[derive(Serialize, Deserialize, Debug)]
struct ProjectCache {
    languages: HashSet<String>,
}

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
    /// Uses a local cache to avoid re-scanning on every run.
    pub fn analyze_project(&self, root: &Path) -> String {
        let cache_dir = root.join(".buddhi");
        let cache_file = cache_dir.join("project.json");

        // 1. Try to load from cache
        if let Ok(contents) = fs::read_to_string(&cache_file) {
            if let Ok(cache) = serde_json::from_str::<ProjectCache>(&contents) {
                tracing::info!("Loaded project languages from cache.");
                return format!(
                    "Detected project language stack (cached): {:?}",
                    cache.languages
                );
            }
        }

        // 2. Cache miss: Run the scanner
        let languages = ProjectScanner::scan(root);

        // 3. Save to cache for next time
        let cache = ProjectCache {
            languages: languages.clone(),
        };
        if fs::create_dir_all(&cache_dir).is_ok() {
            if let Ok(json) = serde_json::to_string(&cache) {
                let _ = fs::write(&cache_file, json);
                tracing::info!("Project languages scanned and cached.");
            }
        }

        format!("Detected project language stack: {:?}", languages)
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
