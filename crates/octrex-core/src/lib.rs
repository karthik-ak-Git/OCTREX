pub mod agent;
pub mod app;
pub mod config;
pub mod db;
pub mod error;
pub mod events;
pub mod hardware;
pub mod ids;
pub mod ipc;
pub mod logging;
pub mod models;
pub mod network;
pub mod permissions;
pub mod privacy;
pub mod providers;
pub mod security;
pub mod services;
pub mod session;
pub mod task;
pub mod workspace;

pub use db::{DatabaseManager, DatabaseState, DatabaseStatus, DbConfig};

pub use agent::{AgentEngine, AgentExecutionRequest, AgentExecutionResponse};
pub use app::{AppLifecycleState, ApplicationState, LifecycleManager};
pub use config::{AppConfig, ProviderConfig, ProviderType};
pub use error::{AppErrorResponse, ErrorCode, OctrexError};
pub use events::{EventBus, EventEnvelope, EventType};
pub use hardware::{
    AcceleratorCategory, Architecture, CompatibilityReason, CompatibilityResult,
    CompatibilityStatus, CpuInfo, DetectionConfidence, GpuInfo, GpuSnapshot, GpuVendor,
    HardwareError, HardwarePerformanceSnapshot, HardwareProfile, HardwareProfiler, HardwareService,
    HardwareSnapshot, HardwareSource, MemoryInfo, ModelCompatibilityEngine, OperatingSystem,
    RealHardwareProbe,
};
pub use ids::{EventId, RequestId, SessionId, TaskId, WorkspaceId};
pub use ipc::{
    ApplicationInfo, BackendHealth, ConfigurationSummary, IpcCommandHandler, RuntimeStatus,
};
pub use models::{
    CallCorrelation, DefaultTokenizer, FinishReason, HardwareRequirement, ModelAvailability,
    ModelCapability, ModelCompatibility, ModelDescriptor, ModelError, ModelId, ModelMessage,
    ModelRegistry, ModelRequest, ModelRequirement, ModelResponse, ModelRuntime, ModelStreamEvent,
    ResponseFormat, TokenCount, Tokenizer, TokenizerInfo, ToolCall, ToolDefinition, Usage,
};
pub use network::{
    classify_ip, is_cloud_metadata_endpoint, AllowlistEntry, Disposition, EvaluatorConfig,
    IpClassification, NetworkAllowlist, NetworkAuditLogger, NetworkAuditRecord, NetworkCapability,
    NetworkConsentRecord, NetworkDecision, NetworkEndpoint, NetworkError, NetworkMode,
    NetworkPolicyBoundary, NetworkPolicyEvaluator, NetworkPolicySet, NetworkProtocol,
    NetworkRequest, NetworkRule, NetworkSecurityService, NetworkSecurityStatus,
    OutboundPayloadBoundary, SafeDnsResolver, SecurityContext, ToolCapabilityDeclaration,
};
pub use privacy::{
    ClassificationConfidence, ClassificationResult, ClassificationSignal, ConsentDecision,
    ConsentManager, ConsentRequest, DecisionFormatter, DecisionState, EffectivePrivacyStatus,
    EvidenceManager, InputSourceType, OutboundPayloadPreview, PolicyAction, PolicyEngine,
    PolicyRule, PolicyScope, PolicySource, PrivacyClassification, PrivacyContext, PrivacyDecision,
    PrivacyGate, PrivacyInput, PrivacyMode, PrivacyRepository, SqlitePrivacyRepository, TrustLevel,
};
pub use providers::{
    ExecutionMode, ModelProvider, ProviderDescriptor, ProviderGateway, ProviderHealth,
    ProviderHealthCheck, ProviderHealthStatus, ProviderId, ProviderRegistry, ProviderStatus,
};
pub use services::{ServiceDescriptor, ServiceRegistry, ServiceStatus};
pub use session::{Session, SessionRegistry, SessionStatus};
pub use task::{Task, TaskRegistry, TaskStatus};
pub use workspace::{FileEntry, WorkspaceInfo, WorkspaceManager, WorkspaceRegistry};

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_full_octrex_foundation_initialization() {
        let app_state = ApplicationState::initialize();
        assert_eq!(
            app_state.lifecycle.current_state(),
            AppLifecycleState::Ready
        );

        let info = IpcCommandHandler::get_application_info();
        assert_eq!(info.name, "OCTREX CODE V4");
        assert_eq!(info.version, "4.0.0");

        let health = IpcCommandHandler::get_backend_health(&app_state);
        assert_eq!(health.status, "HEALTHY");
        assert!(health.initialized);
        assert!(health.subsystems.contains_key("event_bus_service"));

        let summary = IpcCommandHandler::get_configuration_summary(&app_state);
        assert_eq!(summary.active_provider, "opencode");
        assert_eq!(summary.privacy_mode, "strict_local");
    }

    #[tokio::test]
    async fn test_task_creation_and_event_publishing() {
        let app_state = ApplicationState::initialize();
        let mut rx = app_state.event_bus.subscribe();

        let req_id = RequestId::new();
        let task = app_state.task_registry.create_task(
            "Fix bug in backend",
            Some(req_id.clone()),
            None,
            None,
        );
        assert_eq!(task.status, TaskStatus::Created);

        let event = EventEnvelope::new(
            EventType::TaskCreated,
            serde_json::json!({
                "task_id": task.id.as_str(),
                "title": task.title
            }),
        )
        .with_request_id(req_id)
        .with_task_id(task.id.clone());

        app_state.event_bus.publish(event).unwrap();

        let received = rx.recv().await.unwrap();
        assert_eq!(received.event_type, EventType::TaskCreated);
        assert_eq!(received.task_id, Some(task.id));
    }
}
