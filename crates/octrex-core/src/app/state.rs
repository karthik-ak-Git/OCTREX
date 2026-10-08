use crate::agent::AgentEngine;
use crate::app::lifecycle::{AppLifecycleState, LifecycleManager};
use crate::config::AppConfig;
use crate::db::{DatabaseManager, DbConfig};
use crate::events::{EventBus, EventEnvelope, EventType};
use crate::hardware::HardwareService;
use crate::logging::{init_logging, LoggingConfig};
use crate::models::{ModelRegistry, ModelRuntime};
use crate::providers::{ProviderGateway, ProviderRegistry};
use crate::services::{ServiceRegistry, ServiceStatus};
use crate::session::SessionRegistry;
use crate::task::TaskRegistry;
use crate::workspace::WorkspaceRegistry;
use std::sync::{Arc, RwLock};

#[derive(Clone)]
pub struct ApplicationState {
    pub config: Arc<RwLock<AppConfig>>,
    pub lifecycle: Arc<LifecycleManager>,
    pub event_bus: Arc<EventBus>,
    pub service_registry: Arc<ServiceRegistry>,
    pub task_registry: Arc<TaskRegistry>,
    pub session_registry: Arc<SessionRegistry>,
    pub workspace_registry: Arc<WorkspaceRegistry>,
    pub provider_registry: Arc<ProviderRegistry>,
    pub model_registry: Arc<ModelRegistry>,
    pub model_runtime: Arc<ModelRuntime>,
    pub provider_gateway: Arc<ProviderGateway>,
    pub agent_engine: Arc<AgentEngine>,
    pub hardware_service: Arc<HardwareService>,
    pub privacy_gate: Arc<crate::privacy::PrivacyGate>,
    pub policy_engine: Arc<crate::privacy::PolicyEngine>,
    pub network_security: Arc<crate::network::NetworkSecurityService>,
    pub db: Arc<DatabaseManager>,
}

impl ApplicationState {
    pub fn initialize() -> Self {
        Self::initialize_with_db_config(DbConfig::in_memory())
    }

    pub fn initialize_with_db_config(db_config: DbConfig) -> Self {
        let config = AppConfig::load();
        init_logging(&LoggingConfig::default());

        let lifecycle = Arc::new(LifecycleManager::new());
        let event_bus = Arc::new(EventBus::new(2048));
        let service_registry = Arc::new(ServiceRegistry::new());
        let task_registry = Arc::new(TaskRegistry::new());
        let session_registry = Arc::new(SessionRegistry::new());
        let workspace_registry = Arc::new(WorkspaceRegistry::new());

        let provider_registry = Arc::new(ProviderRegistry::new());
        let model_registry = Arc::new(ModelRegistry::new());
        let model_runtime = Arc::new(ModelRuntime::new(
            provider_registry.clone(),
            model_registry.clone(),
        ));
        let provider_gateway = Arc::new(ProviderGateway::with_registries(
            provider_registry.clone(),
            model_registry.clone(),
            model_runtime.clone(),
        ));
        let agent_engine = Arc::new(AgentEngine::new());
        let hardware_service = Arc::new(HardwareService::real().with_event_bus(event_bus.clone()));
        let policy_engine = Arc::new(crate::privacy::PolicyEngine::new());
        let privacy_gate = Arc::new(crate::privacy::PrivacyGate::new());

        let db = Arc::new(DatabaseManager::new(db_config));
        let _ = db.initialize();

        let network_security =
            Arc::new(crate::network::NetworkSecurityService::with_db_and_events(
                db.clone(),
                event_bus.clone(),
            ));

        // Populate Provider & Model Registries from initial config
        if let Ok(handle) = tokio::runtime::Handle::try_current() {
            let gateway = provider_gateway.clone();
            let cfg_clone = config.clone();
            std::thread::scope(|s| {
                s.spawn(move || {
                    handle.block_on(gateway.initialize_from_config(&cfg_clone));
                })
                .join()
                .ok();
            });
        } else if let Ok(rt) = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
        {
            rt.block_on(provider_gateway.initialize_from_config(&config));
        }

        // Register core foundational services
        service_registry
            .register_service("event_bus_service", "In-process async event pub/sub bus");
        service_registry.register_service(
            "workspace_service",
            "Workspace inspection and filesystem manager",
        );
        service_registry
            .register_service("session_service", "Session identity and context registry");
        service_registry.register_service("task_service", "Task lifecycle and correlation manager");
        service_registry.register_service(
            "provider_service",
            "LLM provider gateway and health checker",
        );
        service_registry.register_service(
            "model_registry_service",
            "Normalized model descriptor registry",
        );
        service_registry.register_service(
            "model_runtime_service",
            "Provider-agnostic model runtime execution engine",
        );
        service_registry.register_service(
            "agent_engine_service",
            "Autonomous AI agent execution engine",
        );
        service_registry.register_service(
            "hardware_service",
            "Hardware intelligence and compatibility engine",
        );
        service_registry.register_service(
            "privacy_service",
            "Canonical Octrex Privacy Gate and Data Classifier",
        );
        service_registry.register_service(
            "policy_engine_service",
            "Hierarchical Policy Engine (System > Company > Security > Privacy > User)",
        );
        service_registry.register_service(
            "network_security_service",
            "Centralized Network Security Boundary & Fail-Closed Enforcement Engine",
        );
        service_registry.register_service(
            "database_service",
            "Local embedded SQLite persistence layer",
        );

        // Mark services ready
        service_registry.update_status("event_bus_service", ServiceStatus::Ready);
        service_registry.update_status("workspace_service", ServiceStatus::Ready);
        service_registry.update_status("session_service", ServiceStatus::Ready);
        service_registry.update_status("task_service", ServiceStatus::Ready);
        service_registry.update_status("provider_service", ServiceStatus::Ready);
        service_registry.update_status("model_registry_service", ServiceStatus::Ready);
        service_registry.update_status("model_runtime_service", ServiceStatus::Ready);
        service_registry.update_status("agent_engine_service", ServiceStatus::Ready);
        service_registry.update_status("hardware_service", ServiceStatus::Ready);
        service_registry.update_status("privacy_service", ServiceStatus::Ready);
        service_registry.update_status("policy_engine_service", ServiceStatus::Ready);
        service_registry.update_status("network_security_service", ServiceStatus::Ready);
        service_registry.update_status("database_service", ServiceStatus::Ready);

        lifecycle.set_state(AppLifecycleState::Ready);

        // Publish APPLICATION_READY event
        let ready_event = EventEnvelope::new(
            EventType::ApplicationReady,
            serde_json::json!({
                "app_name": "OCTREX",
                "version": "4.0.0",
                "protocol_version": config.runtime.protocol_version.clone(),
                "status": "READY"
            }),
        );
        let _ = event_bus.publish(ready_event);

        Self {
            config: Arc::new(RwLock::new(config)),
            lifecycle,
            event_bus,
            service_registry,
            task_registry,
            session_registry,
            workspace_registry,
            provider_registry,
            model_registry,
            model_runtime,
            provider_gateway,
            agent_engine,
            hardware_service,
            privacy_gate,
            policy_engine,
            network_security,
            db,
        }
    }

    pub fn shutdown(&self) {
        self.lifecycle.set_state(AppLifecycleState::ShuttingDown);
        let shutdown_event = EventEnvelope::new(
            EventType::ApplicationReady,
            serde_json::json!({ "status": "SHUTTING_DOWN" }),
        );
        let _ = self.event_bus.publish(shutdown_event);
        self.db.shutdown();
        self.lifecycle.set_state(AppLifecycleState::Stopped);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_application_state_initialization() {
        let app_state = ApplicationState::initialize();
        assert!(app_state.lifecycle.is_ready());
        assert!(app_state.service_registry.is_healthy());
    }
}
