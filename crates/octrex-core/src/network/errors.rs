use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error, Clone, Serialize, Deserialize)]
pub enum NetworkError {
    #[error("Invalid endpoint URL: {message}")]
    InvalidEndpoint { message: String },

    #[error("Unsupported protocol: {protocol}")]
    UnsupportedProtocol { protocol: String },

    #[error("Policy unavailable or locked")]
    PolicyUnavailable,

    #[error("Policy evaluation failed: {reason}")]
    PolicyEvaluationFailed { reason: String },

    #[error("Destination blocked by policy: {destination} ({reason})")]
    DestinationBlocked { destination: String, reason: String },

    #[error("Private IP address blocked (SSRF Protection): {address}")]
    PrivateAddressBlocked { address: String },

    #[error("Loopback address blocked: {address}")]
    LoopbackBlocked { address: String },

    #[error("Link-local address blocked (Cloud Metadata Protection): {address}")]
    LinkLocalBlocked { address: String },

    #[error("DNS resolution failed for host '{host}': {reason}")]
    DnsResolutionFailed { host: String, reason: String },

    #[error("Redirect hop blocked: from '{from}' to '{to}' ({reason})")]
    RedirectBlocked {
        from: String,
        to: String,
        reason: String,
    },

    #[error("Consent required for network request to '{destination}'")]
    ConsentRequired { destination: String, reason: String },

    #[error("User consent denied for request to '{destination}'")]
    ConsentDenied { destination: String },

    #[error("Capability denied: {capability}")]
    CapabilityDenied { capability: String },

    #[error("Provider network access denied for '{provider_id}'")]
    ProviderDenied { provider_id: String },

    #[error("Network enforcement is in Disabled mode")]
    NetworkDisabled,

    #[error("Network service unavailable (Fail-Closed)")]
    NetworkUnavailable,

    #[error("Transport error: {message}")]
    TransportFailed { message: String },

    #[error("Request timed out: {message}")]
    Timeout { message: String },
}
