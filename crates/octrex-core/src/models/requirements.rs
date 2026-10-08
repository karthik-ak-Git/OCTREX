use crate::models::capabilities::ModelCapability;
use crate::providers::types::ExecutionMode;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ModelRequirement {
    pub execution_mode: Option<ExecutionMode>,
    pub capabilities: Vec<ModelCapability>,
    pub minimum_context_window: Option<u32>,
    pub vision_required: bool,
    pub tool_calling_required: bool,
    pub structured_output_required: bool,
}

impl ModelRequirement {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_execution_mode(mut self, mode: ExecutionMode) -> Self {
        self.execution_mode = Some(mode);
        self
    }

    pub fn with_capability(mut self, capability: ModelCapability) -> Self {
        if !self.capabilities.contains(&capability) {
            self.capabilities.push(capability);
        }
        self
    }

    pub fn require_vision(mut self) -> Self {
        self.vision_required = true;
        self.with_capability(ModelCapability::Vision)
    }

    pub fn require_tool_calling(mut self) -> Self {
        self.tool_calling_required = true;
        self.with_capability(ModelCapability::ToolCalling)
    }

    pub fn require_structured_output(mut self) -> Self {
        self.structured_output_required = true;
        self.with_capability(ModelCapability::StructuredOutput)
    }

    pub fn matches(&self, descriptor: &crate::models::types::ModelDescriptor) -> bool {
        if let Some(mode) = self.execution_mode {
            if descriptor.execution_mode != mode {
                return false;
            }
        }

        for cap in &self.capabilities {
            if !descriptor.capabilities.contains(cap) {
                return false;
            }
        }

        if let Some(min_context) = self.minimum_context_window {
            match descriptor.context_window {
                Some(ctx) if ctx < min_context => return false,
                None => return false,
                _ => {}
            }
        }

        if self.vision_required && !descriptor.capabilities.contains(&ModelCapability::Vision) {
            return false;
        }

        if self.tool_calling_required
            && !descriptor
                .capabilities
                .contains(&ModelCapability::ToolCalling)
        {
            return false;
        }

        if self.structured_output_required
            && !descriptor
                .capabilities
                .contains(&ModelCapability::StructuredOutput)
            && !descriptor
                .capabilities
                .contains(&ModelCapability::JsonOutput)
        {
            return false;
        }

        true
    }
}
