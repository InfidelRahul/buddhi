use std::collections::HashMap;
use tree_sitter::Language;

/// A central registry that holds all loaded Tree-sitter grammars.
/// This allows Buddhi to dynamically parse any supported language.
pub struct GrammarRegistry {
    grammars: HashMap<String, Language>,
}

impl GrammarRegistry {
    /// Initialize the registry with the core set of supported languages.
    pub fn new() -> Self {
        let mut grammars = HashMap::new();

        // Register Rust
        grammars.insert("rust".to_string(), tree_sitter_rust::language());

        // Register Python
        grammars.insert("python".to_string(), tree_sitter_python::language());

        // Register TypeScript & JavaScript
        grammars.insert(
            "typescript".to_string(),
            tree_sitter_typescript::language_typescript(),
        );
        grammars.insert("javascript".to_string(), tree_sitter_javascript::language());

        // Register Go
        grammars.insert("go".to_string(), tree_sitter_go::language());

        Self { grammars }
    }

    /// Retrieve the Tree-sitter Language parser for a specific language.
    pub fn get_grammar(&self, language: &str) -> Option<&Language> {
        self.grammars.get(language)
    }

    /// Dynamically register a new grammar (for future dynamic loading).
    pub fn register_grammar(&mut self, language: &str, lang: Language) {
        self.grammars.insert(language.to_string(), lang);
    }

    /// Check if a language is supported by this registry.
    pub fn is_supported(&self, language: &str) -> bool {
        self.grammars.contains_key(language)
    }
}
