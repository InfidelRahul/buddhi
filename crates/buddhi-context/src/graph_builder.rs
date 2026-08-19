use buddhi_graph::{CodeGraph, CodeNode, EdgeKind, NodeKind};
use std::path::Path;
use tree_sitter_language_pack::{process, ProcessConfig};

/// Builds a Code Knowledge Graph using the language pack's built-in
/// code intelligence. No manual AST traversal needed.
pub struct GraphBuilder;

impl GraphBuilder {
    pub fn new() -> Self {
        Self
    }

    /// Parses a single file using the language pack's process() API
    /// and adds its structure to the graph.
    pub fn build_from_file(&self, graph: &mut CodeGraph, file_path: &Path, language: &str) {
        let file_path_str = file_path.to_string_lossy().to_string();
        let file_id = format!("file::{}", file_path_str);

        let code = match std::fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(_) => return,
        };

        let file_node = CodeNode {
            id: file_id.clone(),
            name: file_path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| file_path_str.clone()),
            kind: NodeKind::File,
            file_path: file_path_str.clone(),
        };
        graph.add_node(file_node);

        let config = ProcessConfig::new(language).all();
        let result = match process(&code, &config) {
            Ok(r) => r,
            Err(e) => {
                tracing::warn!("Failed to process {}: {}", file_path_str, e);
                return;
            }
        };

        for item in &result.structure {
            // Handle Option<String> name
            let name = match &item.name {
                Some(n) => n.clone(),
                None => continue, // Skip items without names
            };

            let node_id = format!("{}::{}", file_path_str, name);

            // Handle StructureKind enum by converting to string representation
            let kind_str = format!("{:?}", item.kind);
            let kind = match kind_str.to_lowercase().as_str() {
                s if s.contains("function") || s.contains("method") => NodeKind::Function,
                s if s.contains("class")
                    || s.contains("struct")
                    || s.contains("interface")
                    || s.contains("type") =>
                {
                    NodeKind::Class
                }
                s if s.contains("module") || s.contains("namespace") => NodeKind::Module,
                _ => NodeKind::Variable,
            };

            let code_node = CodeNode {
                id: node_id.clone(),
                name,
                kind,
                file_path: file_path_str.clone(),
            };
            graph.add_node(code_node);
            graph.add_edge(&file_id, &node_id, EdgeKind::Contains);
        }
    }
}

impl Default for GraphBuilder {
    fn default() -> Self {
        Self::new()
    }
}
