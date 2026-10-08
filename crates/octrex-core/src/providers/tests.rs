#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::config::{ProviderConfig, ProviderType};
    use crate::providers::{
        adapters::{MockAdapter, OpenAICompatibleAdapter},
        registry::ProviderRegistry,
        types::{ExecutionMode, ModelProvider},
    };

    #[test]
    fn test_provider_registry_lifecycle() {
        let registry = ProviderRegistry::new();
        let mock_p = Arc::new(MockAdapter::new("prov_1", ExecutionMode::Cloud));

        registry.register_provider(mock_p);
        assert_eq!(registry.list_providers().len(), 1);
        assert!(registry.get_provider("prov_1").is_some());

        let removed = registry.unregister_provider("prov_1");
        assert!(removed.is_some());
        assert_eq!(registry.list_providers().len(), 0);
    }

    #[test]
    fn test_security_credentials_not_serialized_in_metadata() {
        let config = ProviderConfig {
            provider_type: ProviderType::Groq,
            api_key: Some("SECRET_GROQ_KEY_12345".to_string()),
            base_url: Some("https://api.groq.com".to_string()),
            default_model: "llama3".to_string(),
        };

        let adapter = OpenAICompatibleAdapter::new("groq", "Groq AI", config);
        let info = adapter.provider_info();

        let json = serde_json::to_string(&info).unwrap();
        assert!(!json.contains("SECRET_GROQ_KEY_12345"));
        assert!(!json.contains("api_key"));
        assert_eq!(info.id, "groq");
    }
}
