use buddhi_core::error::Result;

/// RuleValidator is currently stubbed.
/// Actual AST-based rule validation will be routed through the new
/// buddhi-context CodeLocator (Phase v0.3.0 Step 3).
pub struct RuleValidator;

impl RuleValidator {
    pub fn new() -> Self {
        Self
    }

    pub fn validate(&self, _code: &str, _rules: &[String]) -> Result<bool> {
        // TODO: Wire to buddhi-context GrammarRegistry and CodeLocator
        Ok(true)
    }
}

impl Default for RuleValidator {
    fn default() -> Self {
        Self::new()
    }
}
