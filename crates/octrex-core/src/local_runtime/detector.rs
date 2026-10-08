use crate::local_runtime::adapter::adapter_for_descriptor;
use crate::local_runtime::errors::LocalRuntimeError;
use crate::local_runtime::security::{evaluate_endpoint_against_policy, validate_local_endpoint};
use crate::local_runtime::types::{
    LocalExecutionMode, LocalRuntimeDescriptor, LocalRuntimeHealth, LocalRuntimeType,
};

/// Well-known loopback candidates. These are only probed when the user
/// explicitly triggers discovery — never scanned silently in the background,
/// and never beyond loopback.
pub const DEFAULT_DISCOVERY_CANDIDATES: &[(&str, &str, LocalRuntimeType)] = &[
    (
        "ollama-default",
        "http://127.0.0.1:11434",
        LocalRuntimeType::Ollama,
    ),
    (
        "llamacpp-default",
        "http://127.0.0.1:8080",
        LocalRuntimeType::LlamaCppServer,
    ),
];

/// Explicit discovery request: only the listed endpoints are probed.
#[derive(Debug, Clone)]
pub struct DiscoveryRequest {
    pub endpoint: String,
    pub runtime_type: LocalRuntimeType,
    pub name: Option<String>,
}

impl DiscoveryRequest {
    pub fn new(
        endpoint: impl Into<String>,
        runtime_type: LocalRuntimeType,
        name: Option<String>,
    ) -> Self {
        Self {
            endpoint: endpoint.into(),
            runtime_type,
            name,
        }
    }
}

/// Probe one explicit endpoint and return a descriptor with measured health.
///
/// Fail-closed: invalid endpoints, policy denials, and unreachable hosts all
/// produce `Err` — never an optimistic `Healthy` descriptor.
pub async fn probe_endpoint(
    request: &DiscoveryRequest,
    network_security: &crate::network::NetworkSecurityService,
    runtime_id: Option<String>,
) -> Result<LocalRuntimeDescriptor, LocalRuntimeError> {
    let endpoint = validate_local_endpoint(&request.endpoint)?;
    evaluate_endpoint_against_policy(network_security, &endpoint, "local_runtime_discovery")?;

    let id = runtime_id.unwrap_or_else(|| {
        format!(
            "{}-{}",
            request.runtime_type,
            &uuid::Uuid::new_v4().simple().to_string()[..8]
        )
    });
    let mut descriptor = LocalRuntimeDescriptor {
        id,
        name: request.name.clone().unwrap_or_else(|| {
            format!(
                "Local {} ({})",
                request.runtime_type,
                endpoint.host_and_port()
            )
        }),
        runtime_type: request.runtime_type,
        endpoint: endpoint.raw_url.clone(),
        version: None,
        capabilities: crate::local_runtime::adapter::capabilities_for_type(
            request.runtime_type,
            LocalExecutionMode::Endpoint,
        ),
        execution_mode: LocalExecutionMode::Endpoint,
        health: LocalRuntimeHealth::Unknown,
        last_checked_timestamp: None,
        last_error: None,
        metadata: std::collections::HashMap::new(),
    };

    let adapter = adapter_for_descriptor(&descriptor);
    let report = adapter.probe_health().await;
    descriptor.health = report.health;
    descriptor.last_checked_timestamp = Some(report.checked_at_timestamp);
    descriptor.last_error = report.last_error.clone();
    if report.models_available > 0 {
        descriptor.metadata.insert(
            "discovered_model_count".to_string(),
            report.models_available.to_string(),
        );
    }
    Ok(descriptor)
}

/// Discover runtimes from an explicit candidate list.
///
/// Each candidate is validated through the network boundary before any HTTP
/// request is issued. Unreachable candidates are returned with
/// `Unavailable` health rather than silently dropped, so the UI can explain
/// what was tried and why it failed.
pub async fn discover_runtimes(
    requests: &[DiscoveryRequest],
    network_security: &crate::network::NetworkSecurityService,
) -> Vec<LocalRuntimeDescriptor> {
    let mut out = Vec::new();
    for req in requests {
        match probe_endpoint(req, network_security, None).await {
            Ok(desc) => out.push(desc),
            Err(e) => {
                // Fail-closed but informative: surface the rejected endpoint.
                let mut desc = LocalRuntimeDescriptor::new(
                    format!(
                        "rejected-{}",
                        &uuid::Uuid::new_v4().simple().to_string()[..8]
                    ),
                    req.name.clone().unwrap_or_else(|| req.endpoint.clone()),
                    req.runtime_type,
                    req.endpoint.clone(),
                );
                desc.health = LocalRuntimeHealth::Unavailable;
                desc.last_error = Some(e.to_string());
                out.push(desc);
            }
        }
    }
    out
}

/// Default discovery: the two well-known loopback candidates only.
/// Must only be called from an explicit user action (button/CLI/API call).
pub fn default_discovery_requests() -> Vec<DiscoveryRequest> {
    DEFAULT_DISCOVERY_CANDIDATES
        .iter()
        .map(|(name, endpoint, runtime_type)| {
            DiscoveryRequest::new(*endpoint, *runtime_type, Some(name.to_string()))
        })
        .collect()
}

/// Executable discovery is deliberately NOT implemented: Octrex never
/// executes discovered binaries. Users run their runtime (e.g. `ollama
/// serve`) and register the loopback endpoint explicitly.
pub fn executable_discovery() -> Result<Vec<LocalRuntimeDescriptor>, LocalRuntimeError> {
    Err(LocalRuntimeError::Internal {
        reason: "Executable discovery is disabled by policy: Octrex never executes discovered binaries. Start your local runtime and register its loopback endpoint instead.".to_string(),
    })
}
