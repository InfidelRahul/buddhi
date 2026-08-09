use crate::types::RuleSet;
use dhi_context::tree_sitter::RustSymbolExtractor;
use dhi_core::error::Result;

pub struct RuleValidator {
    extractor: RustSymbolExtractor,
}

impl RuleValidator {
    pub fn try_new() -> Result<Self> {
        Ok(Self {
            extractor: RustSymbolExtractor::try_new()?,
        })
    }

    pub fn validate(&self, source_code: &str, rules: &RuleSet) -> Result<Vec<String>> {
        let mut violations = Vec::new();

        // Use the extractor to parse the AST and retrieve symbols
        // This satisfies dead-code analysis and enables basic rule checking
        let symbols = self.extractor.extract_symbols(source_code)?;

        for rule in &rules.rules {
            for forbidden in &rule.forbid_ast_nodes {
                // Skeleton Logic: Check if any extracted symbol matches the forbidden list.
                // Future Phase: Implement deep AST traversal for method calls like .unwrap()
                if symbols.iter().any(|s| s == forbidden) {
                    violations.push(format!(
                        "Rule '{}' violated: found forbidden symbol '{}'",
                        rule.id, forbidden
                    ));
                }
            }
        }

        Ok(violations)
    }
}
