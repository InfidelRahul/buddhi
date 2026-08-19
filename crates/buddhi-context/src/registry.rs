use std::collections::HashSet;
use std::sync::Mutex;

/// A central registry that delegates all grammar management to
/// tree-sitter-language-pack. Supports 371 languages with on-demand
/// downloading and caching.
pub struct GrammarRegistry {
    verified: Mutex<HashSet<String>>,
}

impl GrammarRegistry {
    pub fn new() -> Self {
        Self {
            verified: Mutex::new(HashSet::new()),
        }
    }

    pub fn is_supported(&self, language: &str) -> bool {
        tree_sitter_language_pack::has_language(language)
    }

    /// Returns the Parser type exported by the language pack itself.
    /// This avoids the "two versions of tree-sitter" type mismatch.
    pub fn get_parser(&self, language: &str) -> Option<tree_sitter_language_pack::Parser> {
        match tree_sitter_language_pack::get_parser(language) {
            Ok(parser) => {
                if let Ok(mut verified) = self.verified.lock() {
                    verified.insert(language.to_string());
                }
                Some(parser)
            }
            Err(e) => {
                tracing::warn!("Failed to load parser for '{}': {}", language, e);
                None
            }
        }
    }

    pub fn verified_languages(&self) -> HashSet<String> {
        self.verified.lock().map(|v| v.clone()).unwrap_or_default()
    }
}

impl Default for GrammarRegistry {
    fn default() -> Self {
        Self::new()
    }
}
