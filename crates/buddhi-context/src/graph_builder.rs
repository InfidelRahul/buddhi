use buddhi_graph::{CodeGraph, CodeNode, EdgeKind, NodeKind};
use std::path::Path;
use tree_sitter_language_pack::{process, ProcessConfig, StructureItem, StructureKind};

/// Builds a Code Knowledge Graph using the official tree-sitter-language-pack API.
pub struct GraphBuilder;

impl GraphBuilder {
    pub fn new() -> Self {
        Self
    }

    /// Processes a single file using the language pack and adds all
    /// extracted intelligence to the graph.
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

        // 1. Add structure nodes (handles nested children recursively)
        self.add_structure_items(graph, &result.structure, &file_path_str, &file_id);

        // 2. Add import edges using `import.source`
        for import in &result.imports {
            if import.source.is_empty() {
                continue;
            }
            let import_id = format!("import::{}", import.source);
            let import_node = CodeNode {
                id: import_id.clone(),
                name: import.source.clone(),
                kind: NodeKind::Module,
                file_path: file_path_str.clone(),
            };
            graph.add_node(import_node);
            graph.add_edge(&file_id, &import_id, EdgeKind::Imports);
        }

        // 3. Add symbol nodes using `symbol.name` (which is a direct String)
        for symbol in &result.symbols {
            if symbol.name.is_empty() {
                continue;
            }
            let symbol_id = format!("{}::symbol::{}", file_path_str, symbol.name);
            let symbol_node = CodeNode {
                id: symbol_id.clone(),
                name: symbol.name.clone(),
                kind: NodeKind::Variable,
                file_path: file_path_str.clone(),
            };
            graph.add_node(symbol_node);
            graph.add_edge(&file_id, &symbol_id, EdgeKind::References);
        }
    }

    /// Recursively adds structure items and their children to the graph.
    fn add_structure_items(
        &self,
        graph: &mut CodeGraph,
        items: &[StructureItem],
        file_path: &str,
        parent_id: &str,
    ) {
        for item in items {
            let name = match &item.name {
                Some(n) => n.clone(),
                None => continue, // Skip anonymous structures
            };

            let node_id = format!("{}::{}", file_path, name);

            // Map the official StructureKind enum to our internal NodeKind
            let kind = match item.kind {
                StructureKind::Function | StructureKind::Method => NodeKind::Function,
                StructureKind::Class
                | StructureKind::Struct
                | StructureKind::Interface
                | StructureKind::Enum
                | StructureKind::Trait => NodeKind::Class,
                StructureKind::Module | StructureKind::Namespace | StructureKind::Impl => {
                    NodeKind::Module
                }
                StructureKind::Other(_) => NodeKind::Variable,
            };

            let code_node = CodeNode {
                id: node_id.clone(),
                name,
                kind,
                file_path: file_path.to_string(),
            };
            graph.add_node(code_node);
            graph.add_edge(parent_id, &node_id, EdgeKind::Contains);

            // Recurse into nested children (e.g., methods inside a class)
            if !item.children.is_empty() {
                self.add_structure_items(graph, &item.children, file_path, &node_id);
            }
        }
    }
}

impl Default for GraphBuilder {
    fn default() -> Self {
        Self::new()
    }
}
