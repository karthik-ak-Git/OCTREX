use crate::models::capabilities::ModelCapability;
use crate::models::types::ModelDescriptor;

pub struct CapabilityEvaluator;

impl CapabilityEvaluator {
    pub fn evaluate_capabilities(
        model: &ModelDescriptor,
        required_caps: &[ModelCapability],
    ) -> (bool, Option<ModelCapability>, String) {
        for cap in required_caps {
            if !model.capabilities.contains(cap) {
                return (
                    false,
                    Some(*cap),
                    format!("Model '{}' lacks required capability '{:?}'", model.id, cap),
                );
            }
        }
        (
            true,
            None,
            "Model satisfies all required capabilities".to_string(),
        )
    }

    pub fn infer_capabilities_for_purpose(purpose: &str) -> Vec<ModelCapability> {
        let mut caps = vec![ModelCapability::TextGeneration];
        let lower = purpose.to_lowercase();

        if lower.contains("code") || lower.contains("refactor") || lower.contains("program") {
            caps.push(ModelCapability::CodeGeneration);
        }
        if lower.contains("vision") || lower.contains("image") || lower.contains("screenshot") {
            caps.push(ModelCapability::Vision);
        }
        if lower.contains("tool") || lower.contains("mcp") || lower.contains("agent") {
            caps.push(ModelCapability::ToolCalling);
        }
        if lower.contains("json") || lower.contains("structured") {
            caps.push(ModelCapability::StructuredOutput);
        }

        caps
    }
}
