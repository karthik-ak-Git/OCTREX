use std::sync::Arc;

use crate::config::{AppConfig, ProviderConfig};
use crate::models::registry::ModelRegistry;
use crate::models::runtime::ModelRuntime;
use crate::providers::adapters::{GoogleAdapter, OllamaAdapter, OpenAICompatibleAdapter};
use crate::providers::health::HealthChecker;
use crate::providers::registry::ProviderRegistry;
use crate::providers::types::ProviderHealthCheck;

pub struct ProviderGateway {
    health_checker: HealthChecker,
    provider_registry: Arc<ProviderRegistry>,
    model_registry: Arc<ModelRegistry>,
    model_runtime: Arc<ModelRuntime>,
}

impl Default for ProviderGateway {
    fn default() -> Self {
        let provider_registry = Arc::new(ProviderRegistry::new());
        let model_registry = Arc::new(ModelRegistry::new());
        let model_runtime = Arc::new(ModelRuntime::new(
            provider_registry.clone(),
            model_registry.clone(),
        ));

        Self {
            health_checker: HealthChecker::new(),
            provider_registry,
            model_registry,
            model_runtime,
        }
    }
}

impl ProviderGateway {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_registries(
        provider_registry: Arc<ProviderRegistry>,
        model_registry: Arc<ModelRegistry>,
        model_runtime: Arc<ModelRuntime>,
    ) -> Self {
        Self {
            health_checker: HealthChecker::new(),
            provider_registry,
            model_registry,
            model_runtime,
        }
    }

    pub fn provider_registry(&self) -> Arc<ProviderRegistry> {
        self.provider_registry.clone()
    }

    pub fn model_registry(&self) -> Arc<ModelRegistry> {
        self.model_registry.clone()
    }

    pub fn model_runtime(&self) -> Arc<ModelRuntime> {
        self.model_runtime.clone()
    }

    /// Sync application config providers into ProviderRegistry and ModelRegistry
    pub async fn initialize_from_config(&self, config: &AppConfig) {
        for (id, pcfg) in &config.providers {
            self.register_from_config(id, pcfg).await;
        }
    }

    pub async fn register_from_config(&self, provider_id: &str, config: &ProviderConfig) {
        let provider_name = match config.provider_type {
            crate::config::ProviderType::OpenCode => "OpenCode AI",
            crate::config::ProviderType::NvidiaNim => "NVIDIA NIM",
            crate::config::ProviderType::Google => "Google Gemini",
            crate::config::ProviderType::Groq => "Groq Cloud",
            crate::config::ProviderType::Ollama => "Local Ollama",
            crate::config::ProviderType::Custom => "Custom Provider",
        };

        let provider_adapter: Arc<dyn crate::providers::types::ModelProvider> =
            match config.provider_type {
                crate::config::ProviderType::Google => Arc::new(GoogleAdapter::new(
                    provider_id,
                    provider_name,
                    config.clone(),
                )),
                crate::config::ProviderType::Ollama => Arc::new(OllamaAdapter::new(
                    provider_id,
                    provider_name,
                    config.clone(),
                )),
                _ => Arc::new(OpenAICompatibleAdapter::new(
                    provider_id,
                    provider_name,
                    config.clone(),
                )),
            };

        self.provider_registry
            .register_provider(provider_adapter.clone());

        for model in provider_adapter.list_models().await {
            self.model_registry.register(model);
        }
    }

    pub async fn check_health(
        &self,
        provider_id: &str,
        config: &ProviderConfig,
    ) -> ProviderHealthCheck {
        self.health_checker.check_health(provider_id, config).await
    }

    pub async fn check_all(&self, config: &AppConfig) -> Vec<ProviderHealthCheck> {
        self.health_checker.check_all(config).await
    }
}
