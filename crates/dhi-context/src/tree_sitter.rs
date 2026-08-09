use dhi_core::error::{DhiError, Result};
use tree_sitter::{Language, Parser};

pub struct RustSymbolExtractor {
    parser: Parser,
}

impl RustSymbolExtractor {
    pub fn try_new() -> Result<Self> {
        let mut parser = Parser::new();
        let language: Language = tree_sitter_rust::language();
        parser
            .set_language(language)
            .map_err(|e| DhiError::Config(format!("Failed to set tree-sitter language: {}", e)))?;
        Ok(Self { parser })
    }

    pub fn extract_symbols(&self, source_code: &str) -> Result<Vec<String>> {
        let tree = self
            .parser
            .parse(source_code, None)
            .ok_or_else(|| DhiError::Config("Tree-sitter parse failed".to_string()))?;

        let mut symbols = Vec::new();
        let root_node = tree.root_node();

        // Simple traversal to find function and struct names
        let mut cursor = root_node.walk();
        for child in root_node.children(&mut cursor) {
            if let Some(node) = child {
                let kind = node.kind();
                if kind == "function_item" || kind == "struct_item" {
                    if let Some(name_node) = node.child_by_field_name("name") {
                        if let Some(name) = source_code.get(name_node.byte_range()) {
                            symbols.push(name.to_string());
                        }
                    }
                }
            }
        }
        Ok(symbols)
    }
}
