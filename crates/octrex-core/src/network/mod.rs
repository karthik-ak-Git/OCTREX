pub mod allowlist;
pub mod audit;
pub mod capabilities;
pub mod decision;
pub mod endpoint;
pub mod errors;
pub mod evaluator;
pub mod gate;
pub mod policy;
pub mod request;
pub mod resolver;
pub mod service;
pub mod types;
pub mod validation;

#[cfg(test)]
mod tests;

pub use allowlist::{AllowlistEntry, NetworkAllowlist};
pub use audit::{NetworkAuditLogger, NetworkAuditRecord};
pub use capabilities::ToolCapabilityDeclaration;
pub use decision::{Disposition, NetworkDecision, PolicySource};
pub use endpoint::NetworkEndpoint;
pub use errors::NetworkError;
pub use evaluator::{EvaluatorConfig, NetworkPolicyEvaluator};
pub use gate::{OutboundPayloadBoundary, SecurityContext};
pub use policy::{NetworkPolicySet, NetworkRule};
pub use request::NetworkRequest;
pub use resolver::SafeDnsResolver;
pub use service::{NetworkConsentRecord, NetworkSecurityService, NetworkSecurityStatus};
pub use types::{IpClassification, NetworkCapability, NetworkMode, NetworkProtocol};
pub use validation::{classify_ip, is_cloud_metadata_endpoint};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NetworkPolicyBoundary {
    pub offline_only: bool,
}

impl Default for NetworkPolicyBoundary {
    fn default() -> Self {
        Self { offline_only: true }
    }
}
