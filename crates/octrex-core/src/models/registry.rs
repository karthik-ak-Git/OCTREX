use std::collections::HashMap;
use std::sync::RwLock;

use crate::models::capabilities::ModelCapability;
use crate::models::requirements::ModelRequirement;
use crate::models::types::ModelDescriptor;
use crate::providers::types::ExecutionMode;

pub struct ModelRegistry {
    models: RwLock<HashMap<String, ModelDescriptor>>,
}

impl Default for ModelRegistry {
    fn default() -> Self {
        Self {
            models: RwLock::new(HashMap::new()),
        }
    }
}

impl ModelRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&self, descriptor: ModelDescriptor) {
        let mut guard = self.models.write().unwrap();
        guard.insert(descriptor.id.clone(), descriptor);
    }

    pub fn unregister(&self, model_id: &str) -> Option<ModelDescriptor> {
        let mut guard = self.models.write().unwrap();
        guard.remove(model_id)
    }

    pub fn get_model(&self, model_id: &str) -> Option<ModelDescriptor> {
        let guard = self.models.read().unwrap();
        guard.get(model_id).cloned()
    }

    pub fn list_models(&self) -> Vec<ModelDescriptor> {
        let guard = self.models.read().unwrap();
        guard.values().cloned().collect()
    }

    pub fn filter_by_capability(&self, cap: ModelCapability) -> Vec<ModelDescriptor> {
        let guard = self.models.read().unwrap();
        guard
            .values()
            .filter(|m| m.capabilities.contains(&cap))
            .cloned()
            .collect()
    }

    pub fn filter_by_execution_mode(&self, mode: ExecutionMode) -> Vec<ModelDescriptor> {
        let guard = self.models.read().unwrap();
        guard
            .values()
            .filter(|m| m.execution_mode == mode)
            .cloned()
            .collect()
    }

    pub fn filter_by_provider(&self, provider_id: &str) -> Vec<ModelDescriptor> {
        let guard = self.models.read().unwrap();
        guard
            .values()
            .filter(|m| m.provider_id == provider_id)
            .cloned()
            .collect()
    }

    pub fn filter_by_requirement(&self, req: &ModelRequirement) -> Vec<ModelDescriptor> {
        let guard = self.models.read().unwrap();
        guard.values().filter(|m| req.matches(m)).cloned().collect()
    }

    pub fn get_local_models(&self) -> Vec<ModelDescriptor> {
        self.filter_by_execution_mode(ExecutionMode::Local)
    }

    pub fn clear(&self) {
        let mut guard = self.models.write().unwrap();
        guard.clear();
    }
}
