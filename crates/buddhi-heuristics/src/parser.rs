use crate::types::HeuristicHints;
use buddhi_core::error::{DhiError, Result};
use buddhi_core::types::TaskType;
use regex::Regex;

pub struct HeuristicParser {
    file_pattern: Regex,
}

impl HeuristicParser {
    pub fn try_new() -> Result<Self> {
        // Detect file paths like src/main.rs or ./file.txt
        let file_pattern = Regex::new(r"(?:^|\s)([a-zA-Z0-9_\-./]+\.[a-zA-Z0-9]+)(?:$|\s)")
            .map_err(|e| DhiError::Config(format!("Regex compilation failed: {}", e)))?;

        Ok(Self { file_pattern })
    }

    pub fn parse(&self, input: &str) -> HeuristicHints {
        let mut hints = HeuristicHints {
            detected_files: Vec::new(),
            detected_symbols: Vec::new(),
            detected_task_type: None,
            detected_constraints: Vec::new(),
        };

        for cap in self.file_pattern.captures_iter(input) {
            if let Some(match_str) = cap.get(1) {
                hints.detected_files.push(match_str.as_str().to_string());
            }
        }

        let lower_input = input.to_lowercase();
        if lower_input.contains("fix") || lower_input.contains("bug") {
            hints.detected_task_type = Some(TaskType::BugFix);
        } else if lower_input.contains("refactor") {
            hints.detected_task_type = Some(TaskType::Refactor);
        } else if lower_input.contains("test") {
            hints.detected_task_type = Some(TaskType::TestGeneration);
        }

        if lower_input.contains("don't break") || lower_input.contains("do not break") {
            hints
                .detected_constraints
                .push("preserve_tests".to_string());
        }

        hints
    }
}
