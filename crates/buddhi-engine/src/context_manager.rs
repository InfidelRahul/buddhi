use buddhi_context::extractor::BuddhiExtractor;
use buddhi_context::graph_builder::GraphBuilder;
use buddhi_context::scanner::ProjectScanner;
use buddhi_graph::{CodeGraph, GraphQuery};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::Path;

#[derive(Serialize, Deserialize, Debug)]
struct ProjectCache {
    languages: HashSet<String>,
}

pub struct ContextManager {
    extractor: BuddhiExtractor,
    graph_builder: GraphBuilder,
    graph: CodeGraph,
}

impl ContextManager {
    pub fn new() -> Self {
        Self {
            extractor: BuddhiExtractor::new(),
            graph_builder: GraphBuilder::new(),
            graph: CodeGraph::new(),
        }
    }

    pub fn analyze_project(&mut self, root: &Path) -> String {
        let cache_dir = root.join(".buddhi");
        let cache_file = cache_dir.join("project.json");

        if let Ok(contents) = fs::read_to_string(&cache_file) {
            if let Ok(cache) = serde_json::from_str::<ProjectCache>(&contents) {
                tracing::info!("Loaded project languages from cache.");
                return format!(
                    "Detected project language stack (cached): {:?}",
                    cache.languages
                );
            }
        }

        let languages = ProjectScanner::scan(root);

        // Build the Code Knowledge Graph
        self.build_graph(root, &languages);
        tracing::info!(
            "Code Knowledge Graph built with {} nodes and {} edges.",
            self.graph.graph.node_count(),
            self.graph.graph.edge_count()
        );

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

    fn build_graph(&mut self, root: &Path, languages: &HashSet<String>) {
        let ext_lang_map: Vec<(&str, &str)> = vec![
            ("rs", "rust"),
            ("py", "python"),
            ("js", "javascript"),
            ("ts", "typescript"),
            ("tsx", "typescript"),
            ("go", "go"),
            ("java", "java"),
            ("rb", "ruby"),
            ("php", "php"),
        ];

        for entry in fs::read_dir(root).into_iter().flatten().flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    if let Some((_, lang)) = ext_lang_map.iter().find(|(e, _)| *e == ext) {
                        if languages.contains(*lang) {
                            self.graph_builder
                                .build_from_file(&mut self.graph, &path, lang);
                        }
                    }
                }
            }
        }
    }

    /// Query the graph for structural context about a specific symbol.
    pub fn query_context(&self, node_id: &str) -> String {
        let query = GraphQuery::new(&self.graph);
        query.summarize_context(node_id)
    }

    /// Find the impact zone of changing a specific node.
    pub fn get_impact_zone(&self, node_id: &str) -> Vec<String> {
        let query = GraphQuery::new(&self.graph);
        query.get_impact_zone(node_id, 3)
    }

    /// Find where a symbol is defined across the project.
    pub fn find_symbol(&self, symbol_name: &str) -> Vec<String> {
        let query = GraphQuery::new(&self.graph);
        query.find_symbol_locations(symbol_name)
    }

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
