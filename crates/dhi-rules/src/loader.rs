use crate::types::RuleSet;
use dhi_core::error::{DhiError, Result};
use std::fs;
use std::path::Path;

pub struct RuleLoader;

impl RuleLoader {
    pub fn load(project_root: &Path) -> Result<RuleSet> {
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
