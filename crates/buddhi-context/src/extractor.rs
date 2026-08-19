use crate::registry::GrammarRegistry;

pub struct BuddhiExtractor {
    registry: GrammarRegistry,
}

impl BuddhiExtractor {
    pub fn new() -> Self {
        Self {
            registry: GrammarRegistry::new(),
        }
    }

    pub fn extract_context(
        &mut self,
        code: &str,
        language: &str,
        search_term: &str,
    ) -> Option<String> {
        let mut parser = self.registry.get_parser(language)?;
        let tree = parser.parse(code)?;
        let root_node = tree.root_node();

        let term = search_term.to_lowercase();
        self.find_matching_node(root_node, &term, code)
    }

    fn find_matching_node(
        &self,
        node: tree_sitter_language_pack::Node,
        term: &str,
        source: &str,
    ) -> Option<String> {
        let kind = node.kind();
        let is_definition = kind.contains("function")
            || kind.contains("class")
            || kind.contains("method")
            || kind.contains("impl")
            || kind.contains("definition");

        if is_definition {
            if let Some(name_node) = node.child_by_field_name("name") {
                let start = name_node.start_byte();
                let end = name_node.end_byte();
                let name = &source[start..end];
                if name.to_lowercase().contains(term) {
                    let node_start = node.start_byte();
                    let node_end = node.end_byte();
                    return Some(source[node_start..node_end].to_string());
                }
            }
        }

        // THE BUDDHI WAY: Cast boundary once, iterate natively, fail safely.
        let child_count: u32 = node
            .child_count()
            .try_into()
            .expect("AST node child count exceeded u32::MAX");

        for i in 0..child_count {
            if let Some(child) = node.child(i) {
                if let Some(found) = self.find_matching_node(child, term, source) {
                    return Some(found);
                }
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
