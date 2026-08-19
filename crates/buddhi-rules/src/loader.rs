use crate::types::{ProjectRule, RuleSet};
use buddhi_core::error::{DhiError, Result};
use std::fs;
use std::path::Path;

pub struct RuleLoader;

impl RuleLoader {
    pub fn load(project_root: &Path) -> Result<RuleSet> {
        let mut ruleset = Self::global_rules();
        let project_rules = Self::load_project_rules(project_root)?;

        // Merge project rules into the global set
        // Global rules are always present; project rules extend them
        ruleset.rules.extend(project_rules.rules);

        Ok(ruleset)
    }

    fn global_rules() -> RuleSet {
        RuleSet {
            rules: vec![
                ProjectRule {
                    id: "global-format".to_string(),
                    description: "Output ONLY valid JSON tool calls. No prose.".to_string(),
                    target_glob: "*".to_string(),
                    forbid_ast_nodes: vec![],
                },
                ProjectRule {
                    id: "global-safety".to_string(),
                    description: "Do not use unwrap() in generated code.".to_string(),
                    target_glob: "*.rs".to_string(),
                    forbid_ast_nodes: vec!["unwrap".to_string()],
                },
            ],
        }
    }

    fn load_project_rules(project_root: &Path) -> Result<RuleSet> {
        let rules_path = project_root.join(".dhi").join("rules.yaml");
        if !rules_path.exists() {
            return Ok(RuleSet { rules: Vec::new() });
        }

        let content = fs::read_to_string(rules_path)
            .map_err(|e| DhiError::Config(format!("Failed to read rules.yaml: {}", e)))?;

        let rules: RuleSet = serde_yaml::from_str(&content)
            .map_err(|e| DhiError::Config(format!("Failed to parse rules.yaml: {}", e)))?;

        Ok(rules)
    }
}
