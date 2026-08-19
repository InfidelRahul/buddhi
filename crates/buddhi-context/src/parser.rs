use crate::registry::GrammarRegistry;
use tree_sitter::Parser;

/// The core AST parser that uses the GrammarRegistry to parse source code.
pub struct ContextParser {
    parser: Parser,
}

impl ContextParser {
    pub fn new() -> Self {
        Self {
            parser: Parser::new(),
        }
    }

    /// Parse a string of source code using the specified language grammar.
    pub fn parse_code(
        &mut self,
        code: &str,
        language: &str,
        registry: &GrammarRegistry,
    ) -> Option<tree_sitter::Tree> {
        if let Some(lang) = registry.get_grammar(language) {
            self.parser.set_language(lang).ok()?;
            self.parser.parse(code, None)
        } else {
            None
        }
    }
}

impl Default for ContextParser {
    fn default() -> Self {
        Self::new()
    }
}
