use crate::providers::registry::ProviderRegistry;
use crate::providers::types::ProviderStatus;
use std::sync::Arc;

pub struct HealthEvaluator {
    provider_registry: Arc<ProviderRegistry>,
}

impl HealthEvaluator {
    pub fn new(provider_registry: Arc<ProviderRegistry>) -> Self {
        Self { provider_registry }
    }

    pub fn evaluate_provider(&self, provider_id: &str) -> (bool, String) {
        match self.provider_registry.get_provider(provider_id) {
            Some(provider) => {
                let info = provider.provider_info();
                if !info.enabled {
                    return (
                        false,
                        format!("Provider '{}' is disabled by configuration", provider_id),
                    );
                }
                match info.status {
                    ProviderStatus::Available => (
                        true,
                        format!("Provider '{}' is online and healthy", provider_id),
                    ),
                    ProviderStatus::Degraded => (
                        true,
                        format!("Provider '{}' is degraded but available", provider_id),
                    ),
                    ProviderStatus::Unavailable => (
                        false,
                        format!("Provider '{}' is currently unavailable", provider_id),
                    ),
                    ProviderStatus::Disabled => {
                        (false, format!("Provider '{}' is disabled", provider_id))
                    }
                    ProviderStatus::Unknown => (
                        false,
                        format!("Provider '{}' health status is unknown", provider_id),
                    ),
                }
            }
            None => (
                false,
                format!(
                    "Provider '{}' is not registered in ProviderRegistry",
                    provider_id
                ),
            ),
        }
    }
}
