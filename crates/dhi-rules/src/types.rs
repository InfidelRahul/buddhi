use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectRule {
    pub id: String,
    pub description: String,
    pub target_glob: String,
    pub forbid_ast_nodes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleSet {
    pub rules: Vec<ProjectRule>,
}
