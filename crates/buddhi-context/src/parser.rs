use crate::registry::GrammarRegistry;

pub struct ContextParser {
    registry: GrammarRegistry,
}

impl ContextParser {
    pub fn new() -> Self {
        Self {
            registry: GrammarRegistry::new(),
        }
    }

    pub fn parse_code(
        &mut self,
        code: &str,
        language: &str,
    ) -> Option<tree_sitter_language_pack::Tree> {
        let mut parser = self.registry.get_parser(language)?;
        parser.parse(code)
    }

    pub fn is_supported(&self, language: &str) -> bool {
        self.registry.is_supported(language)
    }
}

impl Default for ContextParser {
    fn default() -> Self {
        Self::new()
    }
}
