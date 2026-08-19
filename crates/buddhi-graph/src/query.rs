use crate::graph::{CodeGraph, EdgeKind};
use petgraph::graph::NodeIndex;
use petgraph::visit::Bfs;
use std::collections::HashMap;

/// The Graph-RAG Query Engine.
/// Traverses the Code Knowledge Graph to answer structural questions.
pub struct GraphQuery<'a> {
    graph: &'a CodeGraph,
}

impl<'a> GraphQuery<'a> {
    pub fn new(graph: &'a CodeGraph) -> Self {
        Self { graph }
    }

    /// Find a node index by its unique ID.
    fn find_node(&self, node_id: &str) -> Option<NodeIndex> {
        self.graph.node_map.get(node_id).copied()
    }

    /// Get all direct callers of a function (incoming "Calls" edges).
    pub fn get_callers(&self, function_id: &str) -> Vec<String> {
        let Some(idx) = self.find_node(function_id) else {
            return Vec::new();
        };

        self.graph
            .graph
            .neighbors_directed(idx, petgraph::Direction::Incoming)
            .filter_map(|caller_idx| {
                // Check if the edge is a "Calls" edge
                if let Some(edge_idx) = self.graph.graph.find_edge(caller_idx, idx) {
                    if self.graph.graph[edge_idx] == EdgeKind::Calls {
                        return Some(self.graph.graph[caller_idx].id.clone());
                    }
                }
                None
            })
            .collect()
    }

    /// Get all direct dependencies of a node (outgoing edges).
    pub fn get_dependencies(&self, node_id: &str) -> Vec<(EdgeKind, String)> {
        let Some(idx) = self.find_node(node_id) else {
            return Vec::new();
        };

        self.graph
            .graph
            .neighbors_directed(idx, petgraph::Direction::Outgoing)
            .filter_map(|dep_idx| {
                if let Some(edge_idx) = self.graph.graph.find_edge(idx, dep_idx) {
                    let kind = self.graph.graph[edge_idx].clone();
                    let dep_id = self.graph.graph[dep_idx].id.clone();
                    return Some((kind, dep_id));
                }
                None
            })
            .collect()
    }

    /// Get the "impact zone" - all nodes transitively reachable from a given node.
    /// This answers: "If I change this function, what else might break?"
    pub fn get_impact_zone(&self, node_id: &str, max_depth: usize) -> Vec<String> {
        let Some(start_idx) = self.find_node(node_id) else {
            return Vec::new();
        };

        let mut visited = Vec::new();
        let mut bfs = Bfs::new(&self.graph.graph, start_idx);
        let mut depth_map: HashMap<NodeIndex, usize> = HashMap::new();
        depth_map.insert(start_idx, 0);

        while let Some(node_idx) = bfs.next(&self.graph.graph) {
            let current_depth = depth_map.get(&node_idx).copied().unwrap_or(0);
            if current_depth > max_depth {
                continue;
            }

            if node_idx != start_idx {
                visited.push(self.graph.graph[node_idx].id.clone());
            }

            // Propagate depth to neighbors
            for neighbor in self.graph.graph.neighbors(node_idx) {
                if !depth_map.contains_key(&neighbor) {
                    depth_map.insert(neighbor, current_depth + 1);
                }
            }
        }

        visited
    }

    /// Find all files that contain a specific symbol name.
    pub fn find_symbol_locations(&self, symbol_name: &str) -> Vec<String> {
        self.graph
            .graph
            .node_indices()
            .filter_map(|idx| {
                let node = &self.graph.graph[idx];
                if node.name.contains(symbol_name) {
                    Some(node.file_path.clone())
                } else {
                    None
                }
            })
            .collect()
    }

    /// Generate a natural language summary of a node's context for the LLM.
    pub fn summarize_context(&self, node_id: &str) -> String {
        let callers = self.get_callers(node_id);
        let deps = self.get_dependencies(node_id);

        let mut summary = format!("Context for '{}':\n", node_id);

        if !callers.is_empty() {
            summary.push_str(&format!("  Called by: {:?}\n", callers));
        }

        if !deps.is_empty() {
            let dep_names: Vec<String> = deps.iter().map(|(_, id)| id.clone()).collect();
            summary.push_str(&format!("  Depends on: {:?}\n", dep_names));
        }

        if callers.is_empty() && deps.is_empty() {
            summary.push_str("  No structural relationships found.\n");
        }

        summary
    }
}
