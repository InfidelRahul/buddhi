use tree_sitter::{Node, Parser};

/// The "Local Scout" of the Buddhi architecture.
/// It scans the AST to find the exact code chunk (function/class) relevant
/// to the user's intent, stripping away irrelevant boilerplate.
pub struct BuddhiExtractor {
    parser: Parser,
}

impl BuddhiExtractor {
    pub fn new() -> Self {
        Self {
            parser: Parser::new(),
        }
    }

    /// Extracts the relevant code context for a given search term.
    pub fn extract_context(
        &mut self,
        code: &str,
        language: &str,
        search_term: &str,
    ) -> Option<String> {
        // 1. Dynamically load the language grammar from the pack
        let lang = tree_sitter_language_pack::language(language)?;
        self.parser.set_language(&lang).ok()?;

        // 2. Parse the source code into an AST
        let tree = self.parser.parse(code, None)?;
        let root_node = tree.root_node();

        // 3. Search for the matching definition
        let term = search_term.to_lowercase();
        self.find_matching_node(root_node, &term, code)
    }

    /// Recursively traverses the AST to find functions/classes matching the intent.
    fn find_matching_node(&self, node: Node, term: &str, source: &str) -> Option<String> {
        let kind = node.kind();

        // Broadly identify definition nodes across 371 languages
        let is_definition = kind.contains("function")
            || kind.contains("class")
            || kind.contains("method")
            || kind.contains("impl");

        if is_definition {
            // Standard tree-sitter convention: definitions expose a "name" field
            if let Some(name_node) = node.child_by_field_name("name") {
                let name = &source[name_node.byte_range()];
                // Case-insensitive matching against the user's intent
                if name.to_lowercase().contains(term) {
                    // Surgical extraction: Return only the matched node's text
                    return Some(source[node.byte_range()].to_string());
                }
            }
        }

        // Traverse child nodes
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if let Some(found) = self.find_matching_node(child, term, source) {
                return Some(found);
            }
        }
        None
    }
}

impl Default for BuddhiExtractor {
    fn default() -> Self {
        Self::new()
    }
}
