use serde::{Deserialize, Serialize};

/// Represents an optimized user intent after local/cloud inference processing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizedIntent {
    /// The refined intent description
    pub intent: String,
    /// Confidence score between 0.0 and 1.0
    pub confidence: f64,
}

impl OptimizedIntent {
    pub fn new(intent: String, confidence: f64) -> Self {
        Self { intent, confidence }
    }

    /// Check if the intent has high confidence (> 0.8)
    pub fn is_high_confidence(&self) -> bool {
        self.confidence > 0.8
    }
}
