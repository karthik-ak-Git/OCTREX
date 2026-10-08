use serde::{Deserialize, Serialize};
use std::fmt;

use crate::config::ProviderType;
use crate::models::capabilities::ModelCapability;
use crate::models::types::{
    ModelDescriptor, ModelError, ModelRequest, ModelResponse, ModelStreamEvent,
};
use tokio::sync::mpsc;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionMode {
    Local,
    OnPremise,
    Cloud,
}

impl fmt::Display for ExecutionMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExecutionMode::Local => write!(f, "local"),
            ExecutionMode::OnPremise => write!(f, "on_premise"),
            ExecutionMode::Cloud => write!(f, "cloud"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderStatus {
    Available,
    Unavailable,
    Degraded,
    Unknown,
    Disabled,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProviderHealthStatus {
    Connected { models: Vec<String> },
    MissingApiKey,
    AuthError { message: String },
    Offline { reason: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderHealthCheck {
    pub provider_id: String,
    pub status: ProviderHealthStatus,
    pub latency_ms: u128,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderHealth {
    pub provider_id: String,
    pub status: ProviderStatus,
    pub latency_ms: u128,
    pub checked_at_timestamp: u64,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ProviderId(pub String);

impl ProviderId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Provider metadata struct — explicitly DOES NOT contain credentials/API keys.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderDescriptor {
    pub id: String,
    pub name: String,
    pub provider_type: ProviderType,
    pub execution_mode: ExecutionMode,
    pub enabled: bool,
    pub status: ProviderStatus,
    pub base_url: Option<String>,
}

#[async_trait::async_trait]
pub trait ModelProvider: Send + Sync {
    fn provider_info(&self) -> ProviderDescriptor;
    fn capabilities(&self) -> Vec<ModelCapability>;
    async fn list_models(&self) -> Vec<ModelDescriptor>;
    async fn health_check(&self) -> ProviderHealth;
    async fn invoke(&self, request: &ModelRequest) -> Result<ModelResponse, ModelError>;
    async fn stream(
        &self,
        request: &ModelRequest,
    ) -> Result<mpsc::Receiver<ModelStreamEvent>, ModelError>;
}
