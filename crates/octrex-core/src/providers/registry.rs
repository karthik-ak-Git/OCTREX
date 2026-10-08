use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use crate::providers::types::{ModelProvider, ProviderDescriptor, ProviderHealth, ProviderStatus};

pub struct ProviderRegistry {
    providers: RwLock<HashMap<String, Arc<dyn ModelProvider>>>,
}

impl Default for ProviderRegistry {
    fn default() -> Self {
        Self {
            providers: RwLock::new(HashMap::new()),
        }
    }
}

impl ProviderRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_provider(&self, provider: Arc<dyn ModelProvider>) {
        let info = provider.provider_info();
        let mut guard = self.providers.write().unwrap();
        guard.insert(info.id, provider);
    }

    pub fn unregister_provider(&self, provider_id: &str) -> Option<Arc<dyn ModelProvider>> {
        let mut guard = self.providers.write().unwrap();
        guard.remove(provider_id)
    }

    pub fn get_provider(&self, provider_id: &str) -> Option<Arc<dyn ModelProvider>> {
        let guard = self.providers.read().unwrap();
        guard.get(provider_id).cloned()
    }

    pub fn list_providers(&self) -> Vec<ProviderDescriptor> {
        let guard = self.providers.read().unwrap();
        guard.values().map(|p| p.provider_info()).collect()
    }

    pub async fn check_provider_health(&self, provider_id: &str) -> Option<ProviderHealth> {
        let provider = self.get_provider(provider_id)?;
        Some(provider.health_check().await)
    }

    pub async fn check_all_health(&self) -> Vec<ProviderHealth> {
        let providers = {
            let guard = self.providers.read().unwrap();
            guard.values().cloned().collect::<Vec<_>>()
        };

        let mut results = Vec::new();
        for provider in providers {
            results.push(provider.health_check().await);
        }
        results
    }

    pub fn is_provider_available(&self, provider_id: &str) -> bool {
        if let Some(p) = self.get_provider(provider_id) {
            let info = p.provider_info();
            info.enabled && info.status == ProviderStatus::Available
        } else {
            false
        }
    }
}
