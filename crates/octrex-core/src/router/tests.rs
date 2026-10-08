#[cfg(test)]
#[allow(clippy::module_inception)]
mod tests {
    use crate::db::DbConfig;
    use crate::events::EventBus;
    use crate::hardware::{
        fixture_d_24gb_nvidia_workstation, HardwareService, HardwareSnapshot, MockHardwareProbe,
    };
    use crate::ids::RequestId;
    use crate::models::capabilities::ModelCapability;
    use crate::models::registry::ModelRegistry;
    use crate::models::types::{
        HardwareRequirement, ModelAvailability, ModelDescriptor, TokenizerInfo,
    };
    use crate::privacy::{PrivacyClassification, PrivacyGate};
    use crate::providers::adapters::MockAdapter;
    use crate::providers::registry::ProviderRegistry;
    use crate::providers::types::ExecutionMode;
    use crate::router::{
        FallbackGuard, ModelRouter, RoutingDecisionState, RoutingMode, RoutingRequest,
    };
    use std::collections::HashMap;
    use std::sync::Arc;

    fn setup_router() -> (
        ModelRouter,
        Arc<ModelRegistry>,
        Arc<ProviderRegistry>,
        Arc<PrivacyGate>,
    ) {
        let db = Arc::new(crate::db::DatabaseManager::new(DbConfig::in_memory()));
        let _ = db.initialize();
        let event_bus = Arc::new(EventBus::new(100));

        let provider_registry = Arc::new(ProviderRegistry::new());
        let model_registry = Arc::new(ModelRegistry::new());
        let privacy_gate = Arc::new(PrivacyGate::new());
        let hardware_service = {
            let profile = fixture_d_24gb_nvidia_workstation();
            let snapshot = HardwareSnapshot {
                timestamp: 1700000000,
                total_ram_bytes: profile.memory.total_bytes,
                available_ram_bytes: profile.memory.available_bytes.unwrap_or(0),
                ram_usage_percent: 25.0,
                cpu_usage_percent: 10.0,
                gpu_snapshots: vec![],
            };
            Arc::new(HardwareService::new(Arc::new(MockHardwareProbe::new(
                profile, snapshot,
            ))))
        };

        // Register default local provider
        provider_registry.register_provider(Arc::new(MockAdapter::new(
            "local_ollama",
            ExecutionMode::Local,
        )));

        // Register cloud provider
        provider_registry.register_provider(Arc::new(MockAdapter::new(
            "cloud_openai",
            ExecutionMode::Cloud,
        )));

        // Register local model
        model_registry.register(ModelDescriptor {
            id: "llama3-8b".to_string(),
            provider_id: "local_ollama".to_string(),
            model_identifier: "llama3:8b".to_string(),
            display_name: "Llama 3 8B Local".to_string(),
            execution_mode: ExecutionMode::Local,
            capabilities: vec![
                ModelCapability::TextGeneration,
                ModelCapability::CodeGeneration,
                ModelCapability::ToolCalling,
            ],
            context_window: Some(8192),
            max_output_tokens: Some(4096),
            tokenizer: TokenizerInfo::Estimated { factor: 1.0 },
            hardware_requirements: Some(HardwareRequirement {
                minimum_ram: Some(4096),
                recommended_ram: Some(8192),
                minimum_vram: Some(4096),
                recommended_vram: Some(8192),
            }),
            availability: ModelAvailability::Available,
            metadata: HashMap::new(),
        });

        // Register cloud model
        model_registry.register(ModelDescriptor {
            id: "gpt-4o".to_string(),
            provider_id: "cloud_openai".to_string(),
            model_identifier: "gpt-4o".to_string(),
            display_name: "GPT-4o Cloud".to_string(),
            execution_mode: ExecutionMode::Cloud,
            capabilities: vec![
                ModelCapability::TextGeneration,
                ModelCapability::CodeGeneration,
                ModelCapability::Vision,
                ModelCapability::ToolCalling,
                ModelCapability::StructuredOutput,
            ],
            context_window: Some(128000),
            max_output_tokens: Some(4096),
            tokenizer: TokenizerInfo::Exact {
                name: "cl100k_base".to_string(),
            },
            hardware_requirements: None,
            availability: ModelAvailability::Available,
            metadata: HashMap::new(),
        });

        let router = ModelRouter::new(
            model_registry.clone(),
            provider_registry.clone(),
            privacy_gate.clone(),
            hardware_service,
            event_bus,
            db,
        );

        (router, model_registry, provider_registry, privacy_gate)
    }

    #[test]
    fn test_local_model_selection_public_workspace() {
        let (router, _, _, _) = setup_router();
        let req = RoutingRequest::new("coding")
            .with_routing_mode(RoutingMode::LocalOnly)
            .with_privacy_classification(PrivacyClassification::Public);

        let decision = router.route(req);
        assert_eq!(decision.state, RoutingDecisionState::Selected);
        assert_eq!(decision.selected_model_id, Some("llama3-8b".to_string()));
        assert_eq!(decision.execution_mode, Some(ExecutionMode::Local));
    }

    #[test]
    fn test_online_model_selection_public_workspace() {
        let (router, _, _, _) = setup_router();
        let req = RoutingRequest::new("coding")
            .with_routing_mode(RoutingMode::OnlineOnly)
            .with_privacy_classification(PrivacyClassification::Public);

        let decision = router.route(req);
        assert_eq!(decision.state, RoutingDecisionState::Selected);
        assert_eq!(decision.selected_model_id, Some("gpt-4o".to_string()));
        assert_eq!(decision.execution_mode, Some(ExecutionMode::Cloud));
    }

    #[test]
    fn test_confidential_workspace_blocks_online_selection() {
        let (router, _, _, _) = setup_router();
        let req = RoutingRequest::new("coding")
            .with_routing_mode(RoutingMode::OnlineOnly)
            .with_privacy_classification(PrivacyClassification::Confidential);

        let decision = router.route(req);
        assert_ne!(decision.state, RoutingDecisionState::Selected);
        assert!(decision.selected_model_id.is_none());
        assert_eq!(decision.state, RoutingDecisionState::PolicyDenied);
    }

    #[test]
    fn test_no_automatic_cloud_fallback_invariant() {
        let res = FallbackGuard::assert_no_automatic_fallback(RoutingMode::LocalOnly, true);
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("strictly prohibited"));
    }

    #[test]
    fn test_explicit_model_selection_valid() {
        let (router, _, _, _) = setup_router();
        let req = RoutingRequest::new("general")
            .with_user_selected_model("llama3-8b")
            .with_privacy_classification(PrivacyClassification::Public);

        let decision = router.route(req);
        assert_eq!(decision.state, RoutingDecisionState::Selected);
        assert_eq!(decision.selected_model_id, Some("llama3-8b".to_string()));
    }

    #[test]
    fn test_explicit_model_selection_blocked_by_privacy() {
        let (router, _, _, _) = setup_router();
        let req = RoutingRequest::new("general")
            .with_user_selected_model("gpt-4o")
            .with_privacy_classification(PrivacyClassification::Secret);

        let decision = router.route(req);
        assert_eq!(decision.state, RoutingDecisionState::PolicyDenied);
        assert!(decision.selected_model_id.is_none());
    }

    #[test]
    fn test_context_too_large_rejection() {
        let (router, _, _, _) = setup_router();
        let req = RoutingRequest::new("long document")
            .with_routing_mode(RoutingMode::LocalOnly)
            .with_required_context(100000, 4096) // Exceeds llama3-8b (8192)
            .with_privacy_classification(PrivacyClassification::Public);

        let decision = router.route(req);
        assert_eq!(decision.state, RoutingDecisionState::ContextTooLarge);
    }

    #[test]
    fn test_capability_mismatch_filtering() {
        let (router, _, _, _) = setup_router();
        let req = RoutingRequest::new("image processing")
            .with_routing_mode(RoutingMode::LocalOnly)
            .with_capability(ModelCapability::Vision) // llama3-8b lacks vision
            .with_privacy_classification(PrivacyClassification::Public);

        let decision = router.route(req);
        assert_eq!(decision.state, RoutingDecisionState::NoCompatibleModel);
    }

    #[test]
    fn test_preview_evaluation_non_destructive() {
        let (router, _, _, _) = setup_router();
        let req = RoutingRequest::new("test preview").with_routing_mode(RoutingMode::Auto);

        let preview_decision = router.preview(req);
        assert_eq!(preview_decision.state, RoutingDecisionState::Selected);
        assert!(preview_decision.selected_model_id.is_some());
    }
}
