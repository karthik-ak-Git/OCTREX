use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

use crate::providers::types::ExecutionMode;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ModelId(pub String);

impl ModelId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ModelId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelAvailability {
    Available,
    Unavailable,
    Unknown,
    Disabled,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TokenizerInfo {
    Exact { name: String },
    Estimated { factor: f32 },
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct HardwareRequirement {
    pub minimum_ram: Option<u64>,
    pub recommended_ram: Option<u64>,
    pub minimum_vram: Option<u64>,
    pub recommended_vram: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelDescriptor {
    pub id: String,
    pub provider_id: String,
    pub model_identifier: String,
    pub display_name: String,
    pub execution_mode: ExecutionMode,
    pub capabilities: Vec<crate::models::capabilities::ModelCapability>,
    pub context_window: Option<u32>,
    pub max_output_tokens: Option<u32>,
    pub tokenizer: TokenizerInfo,
    pub hardware_requirements: Option<HardwareRequirement>,
    pub availability: ModelAvailability,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelMessage {
    pub role: String,
    pub content: String,
    pub tool_calls: Option<Vec<ToolCall>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ResponseFormat {
    Text,
    JsonObject,
    JsonSchema(serde_json::Value),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallCorrelation {
    pub call_id: String,
    pub request_id: Option<String>,
    pub task_id: Option<String>,
    pub session_id: Option<String>,
}

impl Default for CallCorrelation {
    fn default() -> Self {
        Self {
            call_id: format!("call-{}", uuid::Uuid::new_v4().simple()),
            request_id: None,
            task_id: None,
            session_id: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelRequest {
    pub model_id: String,
    pub messages: Vec<ModelMessage>,
    pub system_instructions: Option<String>,
    pub tools: Vec<ToolDefinition>,
    pub temperature: Option<f32>,
    pub max_output_tokens: Option<u32>,
    pub response_format: ResponseFormat,
    pub metadata: HashMap<String, String>,
    pub correlation: CallCorrelation,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct Usage {
    pub input_tokens: Option<u32>,
    pub output_tokens: Option<u32>,
    pub total_tokens: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FinishReason {
    Stop,
    Length,
    ToolCalls,
    ContentFilter,
    Error,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelResponse {
    pub model_id: String,
    pub content: String,
    pub tool_calls: Vec<ToolCall>,
    pub usage: Usage,
    pub finish_reason: FinishReason,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum ModelStreamEvent {
    TextDelta(String),
    ToolCallDelta {
        index: usize,
        id: Option<String>,
        name: Option<String>,
        arguments: String,
    },
    UsageUpdate(Usage),
    Completed(ModelResponse),
    Failed(String),
}

#[derive(Debug, Clone, thiserror::Error, Serialize, Deserialize, PartialEq)]
pub enum ModelError {
    #[error("Provider '{provider_id}' is unavailable: {reason}")]
    ProviderUnavailable { provider_id: String, reason: String },

    #[error("Model '{model_id}' is unavailable: {reason}")]
    ModelUnavailable { model_id: String, reason: String },

    #[error("Authentication failed for provider '{provider_id}': {message}")]
    AuthenticationFailed {
        provider_id: String,
        message: String,
    },

    #[error("Rate limited by provider '{provider_id}'")]
    RateLimited { provider_id: String },

    #[error("Invalid model request: {message}")]
    InvalidRequest { message: String },

    #[error("Context limit exceeded: requested {requested} tokens, maximum is {max}")]
    ContextTooLarge { requested: u32, max: u32 },

    #[error("Capability '{capability:?}' is unsupported by target model")]
    UnsupportedCapability {
        capability: crate::models::capabilities::ModelCapability,
    },

    #[error("Network error: {message}")]
    NetworkError { message: String },

    #[error("Model request timed out")]
    Timeout,

    #[error("Streaming failed: {message}")]
    StreamFailed { message: String },

    #[error("Provider error ({provider_id}): {message}")]
    ProviderError {
        provider_id: String,
        message: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelCompatibility {
    pub model_id: String,
    pub hardware_compatible: bool,
    pub reason: Option<String>,
}
