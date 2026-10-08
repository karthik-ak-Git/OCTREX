pub mod adapter;
pub mod compatibility;
pub mod detector;
pub mod errors;
pub mod persistence;
pub mod registry;
pub mod security;
pub mod service;
pub mod types;

#[cfg(test)]
pub mod tests;

pub use adapter::{
    adapter_for_descriptor, capabilities_for_type, descriptor_from_discovered,
    initial_state_for_discovery, DiscoveredModel, GenericLocalOpenAiAdapter,
    LlamaCppRuntimeAdapter, LocalRuntimeAdapter, LocalRuntimeProviderBridge, OllamaRuntimeAdapter,
};
pub use compatibility::{estimate_resources, evaluate_local_compatibility};
pub use detector::{
    default_discovery_requests, discover_runtimes, probe_endpoint, DiscoveryRequest,
};
pub use errors::LocalRuntimeError;
pub use persistence::SqliteLocalRuntimeRepository;
pub use registry::{ExecutionSlots, LocalModelStore, LocalRuntimeRegistry};
pub use security::{
    sanitize_model_metadata, validate_download_request, validate_local_endpoint,
    validate_model_file_path, validate_model_format, ALLOWED_DOWNLOAD_SOURCES,
};
pub use service::{
    DownloadModelInput, DownloadSummary, LocalRuntimeConfig, LocalRuntimeService,
    RegisterModelInput,
};
pub use types::{
    LocalAuditEvent, LocalCompatibilityReport, LocalExecutionMode, LocalInferenceMetrics,
    LocalModelRecord, LocalModelState, LocalRuntimeDescriptor, LocalRuntimeHealth,
    LocalRuntimeHealthReport, LocalRuntimeType, LocalStreamEvent, ResourceEstimate,
};
