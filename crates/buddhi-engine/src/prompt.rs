use buddhi_core::types::TaskContract;
use buddhi_memory::store::MemoryStore;
use buddhi_rules::types::RuleSet;

pub struct CloudPromptBuilder;

impl CloudPromptBuilder {
    pub fn build(
        contract: &TaskContract,
        rules: &RuleSet,
        memory: &MemoryStore,
        context: &str,
    ) -> String {
        let mut prompt = String::new();
        prompt.push_str(
            "SYSTEM: You are BUDDHI Cloud Coder. Output ONLY valid JSON tool calls. No prose.\n",
        );
        prompt.push_str(&format!("TASK: {:?}\n", contract.task_type));
        prompt.push_str(&format!("BUDGET: {}\n", contract.token_budget));

        if !rules.rules.is_empty() {
            prompt.push_str("RULES:\n");
            for rule in &rules.rules {
                prompt.push_str(&format!("- {}\n", rule.description));
            }
        }

        let memories = memory.get_relevant("", 3);
        if !memories.is_empty() {
            prompt.push_str("MEMORY:\n");
            for mem in memories {
                prompt.push_str(&format!("- {}\n", mem.lesson));
            }
        }

        prompt.push_str(&format!("CONTEXT:\n{}\n", context));
        prompt
    }
}
