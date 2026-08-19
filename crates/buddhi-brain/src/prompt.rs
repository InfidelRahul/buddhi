use buddhi_heuristics::types::HeuristicHints;

pub struct LocalBrainPromptBuilder;

impl LocalBrainPromptBuilder {
    pub fn build(raw_input: &str, hints: &HeuristicHints) -> String {
        let mut prompt = String::new();
        prompt.push_str("You are BUDDHI local brain. Output only structured intent. Do not explain. Do not write code.\n");
        prompt.push_str("Extract task type, target hints, constraints, risk, privacy, routing.\n");
        prompt.push_str(&format!("Raw task: {}\n", raw_input));

        if !hints.detected_files.is_empty() {
            prompt.push_str(&format!("Detected files: {:?}\n", hints.detected_files));
        }
        if !hints.detected_symbols.is_empty() {
            prompt.push_str(&format!("Detected symbols: {:?}\n", hints.detected_symbols));
        }
        if !hints.detected_constraints.is_empty() {
            prompt.push_str(&format!(
                "Detected constraints: {:?}\n",
                hints.detected_constraints
            ));
        }

        prompt.push_str("Output JSON with fields: task_type, target_file_hints, target_symbol_hints, constraints, risk_level, privacy_level, routing_decision, cloud_instruction_hint.\n");

        prompt
    }
}
