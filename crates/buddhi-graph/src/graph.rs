use petgraph::graph::NodeIndex;
use petgraph::Graph;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// The type of code entity represented in the graph.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum NodeKind {
    File,
    Function,
    Class,
    Module,
    Variable,
}

/// A node in the Code Knowledge Graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeNode {
    pub id: String,   // Unique identifier (e.g., "src/auth.py::login_user")
    pub name: String, // Human-readable name (e.g., "login_user")
    pub kind: NodeKind,
    pub file_path: String,
}

/// The relationship between two code entities.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum EdgeKind {
    Imports,    // File A imports File B
    Calls,      // Function A calls Function B
    Inherits,   // Class A inherits from Class B
    Contains,   // File contains Function
    References, // Variable references a Class
}

/// The in-memory Knowledge Graph for the codebase.
pub struct CodeGraph {
    pub graph: Graph<CodeNode, EdgeKind>,
    pub node_map: HashMap<String, NodeIndex>,
}

impl CodeGraph {
    pub fn new() -> Self {
        Self {
            graph: Graph::new(),
            node_map: HashMap::new(),
        }
    }

    /// Adds a node to the graph and returns its index.
    pub fn add_node(&mut self, node: CodeNode) -> NodeIndex {
        if let Some(&idx) = self.node_map.get(&node.id) {
            return idx;
        }
        let idx = self.graph.add_node(node.clone());
        self.node_map.insert(node.id, idx);
        idx
    }

    /// Adds a directed edge between two nodes.
    pub fn add_edge(&mut self, source_id: &str, target_id: &str, kind: EdgeKind) {
        if let (Some(&source_idx), Some(&target_idx)) =
            (self.node_map.get(source_id), self.node_map.get(target_id))
        {
            self.graph.add_edge(source_idx, target_idx, kind);
        }
    }

    /// Finds all direct dependencies (outgoing edges) of a specific node.
    pub fn get_dependencies(&self, node_id: &str) -> Vec<(EdgeKind, String)> {
        if let Some(&idx) = self.node_map.get(node_id) {
            self.graph
                .neighbors_directed(idx, petgraph::Direction::Outgoing)
                .map(|neighbor_idx| {
                    let edge = self.graph.find_edge(idx, neighbor_idx).unwrap();
                    let edge_kind = self.graph[edge].clone();
                    let neighbor_node = &self.graph[neighbor_idx];
                    (edge_kind, neighbor_node.id.clone())
                })
                .collect()
        } else {
            Vec::new()
        }
    }
}

impl Default for CodeGraph {
    fn default() -> Self {
        Self::new()
    }
}
