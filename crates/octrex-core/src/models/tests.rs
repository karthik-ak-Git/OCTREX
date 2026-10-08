#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Arc;

    use crate::models::{
        capabilities::ModelCapability,
        registry::ModelRegistry,
        requirements::ModelRequirement,
        runtime::{DefaultTokenizer, ModelRuntime, TokenCount, Tokenizer},
        types::*,
    };
    use crate::providers::{
        adapters::MockAdapter, registry::ProviderRegistry, types::ExecutionMode,
    };

    #[test]
    fn test_model_registry_registration_and_filtering() {
        let registry = ModelRegistry::new();

        let m1 = ModelDescriptor {
            id: "local:llama3".to_string(),
            provider_id: "local".to_string(),
            model_identifier: "llama3".to_string(),
            display_name: "Local Llama 3".to_string(),
            execution_mode: ExecutionMode::Local,
            capabilities: vec![
                ModelCapability::TextGeneration,
                ModelCapability::CodeGeneration,
            ],
            context_window: Some(8192),
            max_output_tokens: Some(4096),
            tokenizer: TokenizerInfo::Estimated { factor: 1.0 },
            hardware_requirements: None,
            availability: ModelAvailability::Available,
            metadata: HashMap::new(),
        };

        let m2 = ModelDescriptor {
            id: "google:gemini-1.5-pro".to_string(),
            provider_id: "google".to_string(),
            model_identifier: "gemini-1.5-pro".to_string(),
            display_name: "Gemini Pro".to_string(),
            execution_mode: ExecutionMode::Cloud,
            capabilities: vec![
                ModelCapability::TextGeneration,
                ModelCapability::Vision,
                ModelCapability::ToolCalling,
            ],
            context_window: Some(2_000_000),
            max_output_tokens: Some(8192),
            tokenizer: TokenizerInfo::Estimated { factor: 1.0 },
            hardware_requirements: None,
            availability: ModelAvailability::Available,
            metadata: HashMap::new(),
        };

        registry.register(m1.clone());
        registry.register(m2.clone());

        assert_eq!(registry.list_models().len(), 2);
        assert_eq!(
            registry.get_model("local:llama3").unwrap().display_name,
            "Local Llama 3"
        );

        // Filter by capability
        let vision_models = registry.filter_by_capability(ModelCapability::Vision);
        assert_eq!(vision_models.len(), 1);
        assert_eq!(vision_models[0].id, "google:gemini-1.5-pro");

        // Filter by execution mode
        let local_models = registry.get_local_models();
        assert_eq!(local_models.len(), 1);
        assert_eq!(local_models[0].id, "local:llama3");

        // Filter by requirement
        let req = ModelRequirement::new()
            .with_execution_mode(ExecutionMode::Cloud)
            .require_vision();
        let matched = registry.filter_by_requirement(&req);
        assert_eq!(matched.len(), 1);
        assert_eq!(matched[0].id, "google:gemini-1.5-pro");
    }

    #[test]
    fn test_tokenizer_token_counting() {
        let tokenizer = DefaultTokenizer;

        let empty_count = tokenizer.count_tokens("");
        assert_eq!(empty_count, TokenCount::Exact(0));

        let text_count = tokenizer.count_tokens("Hello world from Octrex engine!");
        assert!(matches!(text_count, TokenCount::Estimated(_)));
        if let TokenCount::Estimated(n) = text_count {
            assert!(n > 0);
        }

        let messages = vec![ModelMessage {
            role: "user".to_string(),
            content: "Write a function in Rust".to_string(),
            tool_calls: None,
        }];
        let msg_count = tokenizer.count_messages(&messages);
        assert!(matches!(msg_count, TokenCount::Estimated(_)));
    }

    #[tokio::test]
    async fn test_model_runtime_invocation() {
        let p_registry = Arc::new(ProviderRegistry::new());
        let m_registry = Arc::new(ModelRegistry::new());

        let mock_provider = Arc::new(MockAdapter::new("mock_p", ExecutionMode::Local));
        p_registry.register_provider(mock_provider.clone());

        let mock_model = ModelDescriptor {
            id: "mock_p:mock-model".to_string(),
            provider_id: "mock_p".to_string(),
            model_identifier: "mock-model".to_string(),
            display_name: "Mock Model".to_string(),
            execution_mode: ExecutionMode::Local,
            capabilities: vec![ModelCapability::TextGeneration],
            context_window: Some(4096),
            max_output_tokens: Some(1024),
            tokenizer: TokenizerInfo::Estimated { factor: 1.0 },
            hardware_requirements: None,
            availability: ModelAvailability::Available,
            metadata: HashMap::new(),
        };
        m_registry.register(mock_model);

        let runtime = ModelRuntime::new(p_registry, m_registry);

        let req = ModelRequest {
            model_id: "mock_p:mock-model".to_string(),
            messages: vec![ModelMessage {
                role: "user".to_string(),
                content: "Test prompt".to_string(),
                tool_calls: None,
            }],
            system_instructions: None,
            tools: vec![],
            temperature: None,
            max_output_tokens: None,
            response_format: ResponseFormat::Text,
            metadata: HashMap::new(),
            correlation: CallCorrelation::default(),
        };

        let resp = runtime.invoke(req).await.unwrap();
        assert_eq!(resp.model_id, "mock_p:mock-model");
        assert!(resp.content.contains("Mock response to: Test prompt"));
    }
}
