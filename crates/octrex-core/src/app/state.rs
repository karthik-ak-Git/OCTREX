use crate::agent::AgentEngine;
use crate::app::lifecycle::{AppLifecycleState, LifecycleManager};
use crate::config::AppConfig;
use crate::db::{DatabaseManager, DbConfig};
use crate::events::{EventBus, EventEnvelope, EventType};
use crate::hardware::HardwareService;
use crate::local_runtime::{LocalRuntimeConfig, LocalRuntimeService};
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
    pub filesystem_security: Arc<crate::filesystem::FilesystemSecurityService>,
    pub tool_registry: Arc<crate::tools::ToolRegistry>,
    pub mcp_registry: Arc<crate::tools::McpRegistry>,
    pub tool_runtime: Arc<crate::tools::ToolRuntime>,
    pub context_engine: Arc<crate::context::ContextService>,
    pub orchestration_service: Arc<crate::orchestration::OrchestrationService>,
    pub model_router: Arc<crate::router::ModelRouter>,
    pub verification_engine: Arc<crate::verification::VerificationEngine>,
    pub skill_service: Arc<crate::skills::SkillService>,
    pub workflow_service: Arc<crate::workflows::WorkflowService>,
    pub memory_service: Arc<crate::memory::MemoryService>,
    pub document_service: Arc<crate::documents::DocumentService>,
    pub local_runtime: Arc<LocalRuntimeService>,
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

        let filesystem_security = Arc::new(crate::filesystem::FilesystemSecurityService::new(
            db.clone(),
            event_bus.clone(),
            privacy_gate.clone(),
            network_security.clone(),
        ));

        let tool_registry = Arc::new(crate::tools::ToolRegistry::new());
        let mcp_registry = Arc::new(crate::tools::McpRegistry::new());
        let tool_runtime = Arc::new(crate::tools::ToolRuntime::new(
            tool_registry.clone(),
            mcp_registry.clone(),
            event_bus.clone(),
            db.clone(),
            network_security.clone(),
            privacy_gate.clone(),
        ));

        let context_engine = Arc::new(crate::context::ContextService::new(
            event_bus.clone(),
            db.clone(),
        ));

        let model_router = Arc::new(crate::router::ModelRouter::new(
            model_registry.clone(),
            provider_registry.clone(),
            privacy_gate.clone(),
            hardware_service.clone(),
            event_bus.clone(),
            db.clone(),
        ));

        let verification_engine = Arc::new(crate::verification::VerificationEngine::new(
            db.clone(),
            event_bus.clone(),
            filesystem_security.clone(),
            network_security.clone(),
            privacy_gate.clone(),
            context_engine.clone(),
            model_registry.clone(),
            model_runtime.clone(),
        ));
        verification_engine.attach_router(model_router.clone());

        let orchestration_service = Arc::new(
            crate::orchestration::OrchestrationService::new(
                crate::orchestration::limits::OrchestratorLimits::default(),
                task_registry.clone(),
                db.clone(),
                event_bus.clone(),
                model_runtime.clone(),
                model_registry.clone(),
                tool_runtime.clone(),
                filesystem_security.clone(),
                context_engine.clone(),
                privacy_gate.clone(),
            )
            .with_verification_engine(verification_engine.clone()),
        );

        let skill_service = Arc::new(crate::skills::SkillService::new(
            db.clone(),
            event_bus.clone(),
        ));
        skill_service.seed_builtins();

        let workflow_service = Arc::new(crate::workflows::WorkflowService::new(
            db.clone(),
            event_bus.clone(),
        ));

        let memory_service = Arc::new(crate::memory::MemoryService::new(
            db.clone(),
            event_bus.clone(),
        ));

        let document_service = Arc::new(crate::documents::DocumentService::new(
            db.clone(),
            event_bus.clone(),
            filesystem_security.clone(),
            privacy_gate.clone(),
            network_security.clone(),
            context_engine.clone(),
            model_router.clone(),
            model_runtime.clone(),
            verification_engine.clone(),
        ));

        let local_runtime = Arc::new(LocalRuntimeService::new(
            LocalRuntimeConfig::default(),
            model_registry.clone(),
            provider_registry.clone(),
            hardware_service.clone(),
            network_security.clone(),
            privacy_gate.clone(),
            filesystem_security.clone(),
            event_bus.clone(),
            db.clone(),
        ));
        local_runtime.restore_from_db();

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
            "filesystem_security_service",
            "Authoritative Local Filesystem Security Boundary & Sandbox Engine",
        );
        service_registry.register_service(
            "tool_runtime_service",
            "Centralized Tool Runtime & MCP Security Execution Governance Engine",
        );
        service_registry.register_service(
            "context_engine_service",
            "Centralized Production-Grade Context Engine, Token Budgeting & Compaction",
        );
        service_registry.register_service(
            "orchestration_service",
            "Production Agent Orchestration, Planning, Execution Loop & Recovery Engine",
        );
        service_registry.register_service(
            "model_router_service",
            "Production Model Router, Authorization-Aware Selection & Policy Engine",
        );
        service_registry.register_service(
            "skill_service",
            "Declarative Skill Registry, Validation, Matching & Versioned Execution",
        );
        service_registry.register_service(
            "workflow_service",
            "Declarative Workflow Registry, DAG Validation & Orchestrator-Delegated Execution",
        );
        service_registry.register_service(
            "memory_service",
            "Scoped Agent Memory, Classification, Provenance, Retention & Poisoning Defense",
        );
        service_registry.register_service(
            "document_service",
            "Local-First Document Intelligence & Artifact Pipeline (Intake, Parsing, Chunking, Retrieval, Lineage)",
        );
        service_registry.register_service(
            "local_runtime_service",
            "Local Runtime & Model Management: discovery, lifecycle, guarded local inference",
        );
        service_registry.register_service(
            "verification_engine_service",
            "Verification & Reliability Engine, Evidence Checks & Completion Gate",
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
        service_registry.update_status("filesystem_security_service", ServiceStatus::Ready);
        service_registry.update_status("tool_runtime_service", ServiceStatus::Ready);
        service_registry.update_status("context_engine_service", ServiceStatus::Ready);
        service_registry.update_status("orchestration_service", ServiceStatus::Ready);
        service_registry.update_status("model_router_service", ServiceStatus::Ready);
        service_registry.update_status("verification_engine_service", ServiceStatus::Ready);
        service_registry.update_status("skill_service", ServiceStatus::Ready);
        service_registry.update_status("workflow_service", ServiceStatus::Ready);
        service_registry.update_status("memory_service", ServiceStatus::Ready);
        service_registry.update_status("document_service", ServiceStatus::Ready);
        service_registry.update_status("local_runtime_service", ServiceStatus::Ready);
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
            filesystem_security,
            tool_registry,
            mcp_registry,
            tool_runtime,
            context_engine,
            orchestration_service,
            model_router,
            verification_engine,
            skill_service,
            workflow_service,
            memory_service,
            document_service,
            local_runtime,
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
