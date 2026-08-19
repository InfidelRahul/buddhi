use buddhi_context::extractor::BuddhiExtractor;
use buddhi_context::graph_builder::GraphBuilder;
use buddhi_context::scanner::ProjectScanner;
use buddhi_graph::{CodeGraph, GraphQuery};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::Path;
use tree_sitter_language_pack::{process, ProcessConfig};

#[derive(Serialize, Deserialize, Debug)]
struct ProjectCache {
    languages: HashSet<String>,
}

/// Orchestrates the Local Scout: scanning, graph building, and RAG context.
/// Leverages tree-sitter-language-pack for ALL code intelligence.
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

    /// Scans project, builds graph, caches results.
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
        self.build_graph(root, &languages);
        tracing::info!(
            "Code Knowledge Graph: {} nodes, {} edges.",
            self.graph.graph.node_count(),
            self.graph.graph.edge_count()
        );

        let cache = ProjectCache {
            languages: languages.clone(),
        };
        if fs::create_dir_all(&cache_dir).is_ok() {
            if let Ok(json) = serde_json::to_string(&cache) {
                let _ = fs::write(&cache_file, json);
            }
        }

        format!("Detected project language stack: {:?}", languages)
    }

    /// Builds Graph-RAG context using the language pack's syntax-aware chunks.
    /// 1. Query graph for relevant files/symbols matching the task
    /// 2. Use language pack's process() to get syntax-aware chunks for those files
    /// 3. Return chunks as RAG context for the LLM
    pub fn build_rag_context(&self, task: &str, root: &Path) -> String {
        let query = GraphQuery::new(&self.graph);
        let mut context_parts: Vec<String> = Vec::new();

        // Extract keywords from task
        let keywords: Vec<&str> = task
            .split(|c: char| !c.is_alphanumeric() && c != '_')
            .filter(|w| w.len() > 3)
            .collect();

        // Find relevant files via graph symbol search
        let mut relevant_files = HashSet::new();
        for keyword in &keywords {
            for file in query.find_symbol_locations(keyword) {
                relevant_files.insert(file);
            }
        }

        if relevant_files.is_empty() {
            return String::from("No structural context matched for this task.");
        }

        context_parts.push(format!(
            "Relevant files: [{}]",
            relevant_files
                .iter()
                .cloned()
                .collect::<Vec<_>>()
                .join(", ")
        ));

        // Use language pack's process() to get syntax-aware chunks for each file
        // This is the RAG/LLM pipeline feature built into the language pack
        for file_path in &relevant_files {
            let full_path = root.join(file_path);
            let code = match fs::read_to_string(&full_path) {
                Ok(c) => c,
                Err(_) => continue,
            };

            // Detect language from extension
            let lang = self.detect_language_from_path(file_path);
            let config = ProcessConfig::new(&lang).all();

            match process(&code, &config) {
                Ok(result) => {
                    // Use the language pack's syntax-aware chunks directly
                    // These are pre-optimized for RAG/LLM pipelines
                    if !result.chunks.is_empty() {
                        context_parts.push(format!(
                            "--- {} ({} chunks) ---",
                            file_path,
                            result.chunks.len()
                        ));
                        for chunk in &result.chunks {
                            context_parts.push(chunk.content.clone());
                        }
                    }

                    // Also include symbol summary for quick reference
                    if !result.symbols.is_empty() {
                        let symbol_names: Vec<&str> =
                            result.symbols.iter().map(|s| s.name.as_str()).collect();
                        context_parts.push(format!(
                            "Symbols in {}: [{}]",
                            file_path,
                            symbol_names.join(", ")
                        ));
                    }
                }
                Err(e) => {
                    tracing::warn!("Failed to process {}: {}", file_path, e);
                }
            }
        }

        format!("Graph-RAG Context:\n{}", context_parts.join("\n"))
    }

    fn detect_language_from_path(&self, path: &str) -> String {
        let ext = path.rsplit('.').next().unwrap_or("");
        match ext {
            "rs" => "rust",
            "py" => "python",
            "js" | "mjs" | "cjs" => "javascript",
            "ts" | "tsx" => "typescript",
            "go" => "go",
            "java" => "java",
            "rb" => "ruby",
            "php" => "php",
            "c" | "h" => "c",
            "cpp" | "cc" | "hpp" => "cpp",
            "cs" => "c_sharp",
            "swift" => "swift",
            "kt" | "kts" => "kotlin",
            "scala" => "scala",
            "sh" | "bash" => "bash",
            _ => "text",
        }
        .to_string()
    }

    fn build_graph(&mut self, root: &Path, languages: &HashSet<String>) {
        let ext_lang_map: [(&str, &str); 9] = [
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

        let Ok(entries) = fs::read_dir(root) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            let Some(ext) = path.extension().and_then(|e| e.to_str()) else {
                continue;
            };
            let Some((_, lang)) = ext_lang_map.iter().find(|(e, _)| *e == ext) else {
                continue;
            };
            if languages.contains(*lang) {
                self.graph_builder
                    .build_from_file(&mut self.graph, &path, lang);
            }
        }
    }

    pub fn extract_code_chunk(
        &mut self,
        root: &Path,
        file_path: &str,
        language: &str,
        search_term: &str,
    ) -> Option<String> {
        let full_path = root.join(file_path);
        let code = fs::read_to_string(&full_path).ok()?;
        self.extractor.extract_context(&code, language, search_term)
    }
}

impl Default for ContextManager {
    fn default() -> Self {
        Self::new()
    }
}
