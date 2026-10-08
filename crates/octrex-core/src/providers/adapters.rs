use reqwest::Client;
use serde::Deserialize;
use std::collections::HashMap;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::sync::mpsc;

use crate::config::{ProviderConfig, ProviderType};
use crate::models::capabilities::ModelCapability;
use crate::models::types::{
    FinishReason, HardwareRequirement, ModelAvailability, ModelDescriptor, ModelError,
    ModelRequest, ModelResponse, ModelStreamEvent, TokenizerInfo, Usage,
};
use crate::providers::types::{
    ExecutionMode, ModelProvider, ProviderDescriptor, ProviderHealth, ProviderStatus,
};

fn now_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

// ============================================================================
// OPENAI COMPATIBLE ADAPTER (NVIDIA NIM, GROQ, OPENCODE, CUSTOM)
// ============================================================================
pub struct OpenAICompatibleAdapter {
    id: String,
    name: String,
    provider_type: ProviderType,
    execution_mode: ExecutionMode,
    config: ProviderConfig,
    http_client: Client,
}

impl OpenAICompatibleAdapter {
    pub fn new(id: impl Into<String>, name: impl Into<String>, config: ProviderConfig) -> Self {
        let http_client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .unwrap_or_else(|_| Client::new());

        let execution_mode = match config.provider_type {
            ProviderType::Ollama => ExecutionMode::Local,
            _ => ExecutionMode::Cloud,
        };

        Self {
            id: id.into(),
            name: name.into(),
            provider_type: config.provider_type.clone(),
            execution_mode,
            config,
            http_client,
        }
    }

    fn default_models(&self) -> Vec<ModelDescriptor> {
        let models = match self.provider_type {
            ProviderType::OpenCode => vec![
                ("opencode-free-router", "OpenCode Free Router"),
                ("qwen2.5-coder-32b-free", "Qwen 2.5 Coder 32B (Free)"),
                ("deepseek-r1-free", "DeepSeek R1 (Free)"),
                ("llama-3.3-70b-free", "Llama 3.3 70B (Free)"),
            ],
            ProviderType::NvidiaNim => vec![
                ("meta/llama-3.3-70b-instruct", "Meta Llama 3.3 70B Instruct"),
                (
                    "mistralai/mistral-large-2-instruct",
                    "Mistral Large 2 Instruct",
                ),
            ],
            ProviderType::Groq => vec![
                ("llama-3.3-70b-versatile", "Groq Llama 3.3 70B Versatile"),
                ("mixtral-8x7b-32768", "Groq Mixtral 8x7B"),
            ],
            _ => vec![(
                self.config.default_model.as_str(),
                self.config.default_model.as_str(),
            )],
        };

        models
            .into_iter()
            .map(|(id_str, name_str)| ModelDescriptor {
                id: format!("{}:{}", self.id, id_str),
                provider_id: self.id.clone(),
                model_identifier: id_str.to_string(),
                display_name: name_str.to_string(),
                execution_mode: self.execution_mode,
                capabilities: vec![
                    ModelCapability::TextGeneration,
                    ModelCapability::CodeGeneration,
                    ModelCapability::Streaming,
                    ModelCapability::ToolCalling,
                ],
                context_window: Some(128_000),
                max_output_tokens: Some(4096),
                tokenizer: TokenizerInfo::Estimated { factor: 1.0 },
                hardware_requirements: None,
                availability: ModelAvailability::Available,
                metadata: HashMap::new(),
            })
            .collect()
    }
}

#[async_trait::async_trait]
impl ModelProvider for OpenAICompatibleAdapter {
    fn provider_info(&self) -> ProviderDescriptor {
        ProviderDescriptor {
            id: self.id.clone(),
            name: self.name.clone(),
            provider_type: self.provider_type.clone(),
            execution_mode: self.execution_mode,
            enabled: true,
            status: ProviderStatus::Available,
            base_url: self.config.base_url.clone(),
        }
    }

    fn capabilities(&self) -> Vec<ModelCapability> {
        vec![
            ModelCapability::TextGeneration,
            ModelCapability::CodeGeneration,
            ModelCapability::Streaming,
            ModelCapability::ToolCalling,
            ModelCapability::StructuredOutput,
        ]
    }

    async fn list_models(&self) -> Vec<ModelDescriptor> {
        self.default_models()
    }

    async fn health_check(&self) -> ProviderHealth {
        let start = Instant::now();
        let api_key = match &self.config.api_key {
            Some(key) if !key.trim().is_empty() => key.trim(),
            _ => {
                if self.provider_type != ProviderType::OpenCode {
                    return ProviderHealth {
                        provider_id: self.id.clone(),
                        status: ProviderStatus::Unavailable,
                        latency_ms: start.elapsed().as_millis(),
                        checked_at_timestamp: now_timestamp(),
                        error_message: Some("Missing API key".to_string()),
                    };
                }
                "free-opencode-router"
            }
        };

        let base_url = self
            .config
            .base_url
            .as_deref()
            .unwrap_or("https://api.openai.com/v1");
        let endpoint = format!("{}/models", base_url.trim_end_matches('/'));

        match self
            .http_client
            .get(&endpoint)
            .bearer_auth(api_key)
            .send()
            .await
        {
            Ok(resp) => {
                let status = if resp.status().is_success() {
                    ProviderStatus::Available
                } else if resp.status().as_u16() == 401 || resp.status().as_u16() == 403 {
                    ProviderStatus::Unavailable
                } else {
                    ProviderStatus::Degraded
                };

                ProviderHealth {
                    provider_id: self.id.clone(),
                    status,
                    latency_ms: start.elapsed().as_millis(),
                    checked_at_timestamp: now_timestamp(),
                    error_message: if status == ProviderStatus::Available {
                        None
                    } else {
                        Some(format!("HTTP {}", resp.status()))
                    },
                }
            }
            Err(e) => ProviderHealth {
                provider_id: self.id.clone(),
                status: ProviderStatus::Unavailable,
                latency_ms: start.elapsed().as_millis(),
                checked_at_timestamp: now_timestamp(),
                error_message: Some(e.to_string()),
            },
        }
    }

    async fn invoke(&self, request: &ModelRequest) -> Result<ModelResponse, ModelError> {
        let api_key = self
            .config
            .api_key
            .as_ref()
            .filter(|k| !k.trim().is_empty())
            .map(|s| s.as_str())
            .unwrap_or_else(|| {
                if self.provider_type == ProviderType::OpenCode {
                    "free-opencode-router"
                } else {
                    ""
                }
            });

        if api_key.is_empty() {
            return Err(ModelError::AuthenticationFailed {
                provider_id: self.id.clone(),
                message: "API Key is missing for provider".to_string(),
            });
        }

        let base_url = self
            .config
            .base_url
            .as_deref()
            .unwrap_or("https://api.openai.com/v1");
        let endpoint = format!("{}/chat/completions", base_url.trim_end_matches('/'));

        let target_model = request
            .model_id
            .split_once(':')
            .map(|(_, m)| m)
            .unwrap_or(&request.model_id);

        let mut messages = Vec::new();

        if let Some(sys) = &request.system_instructions {
            messages.push(serde_json::json!({
                "role": "system",
                "content": sys
            }));
        }

        for msg in &request.messages {
            messages.push(serde_json::json!({
                "role": msg.role,
                "content": msg.content
            }));
        }

        let mut body = serde_json::json!({
            "model": target_model,
            "messages": messages,
            "temperature": request.temperature.unwrap_or(0.3),
        });

        if let Some(max_tokens) = request.max_output_tokens {
            body["max_tokens"] = serde_json::json!(max_tokens);
        }

        let res = self
            .http_client
            .post(&endpoint)
            .bearer_auth(api_key)
            .json(&body)
            .send()
            .await
            .map_err(|e| ModelError::NetworkError {
                message: e.to_string(),
            })?;

        let status_code = res.status();
        if status_code.as_u16() == 401 || status_code.as_u16() == 403 {
            return Err(ModelError::AuthenticationFailed {
                provider_id: self.id.clone(),
                message: format!("Authentication failed (HTTP {})", status_code),
            });
        }
        if status_code.as_u16() == 429 {
            return Err(ModelError::RateLimited {
                provider_id: self.id.clone(),
            });
        }
        if !status_code.is_success() {
            let err_text = res.text().await.unwrap_or_default();
            return Err(ModelError::ProviderError {
                provider_id: self.id.clone(),
                message: format!("Provider error HTTP {}: {}", status_code, err_text),
            });
        }

        #[derive(Deserialize)]
        struct ChoiceMessage {
            content: Option<String>,
        }
        #[derive(Deserialize)]
        struct Choice {
            message: ChoiceMessage,
            finish_reason: Option<String>,
        }
        #[derive(Deserialize)]
        struct UsageData {
            prompt_tokens: Option<u32>,
            completion_tokens: Option<u32>,
            total_tokens: Option<u32>,
        }
        #[derive(Deserialize)]
        struct ChatCompletionResponse {
            choices: Vec<Choice>,
            usage: Option<UsageData>,
        }

        let parsed: ChatCompletionResponse =
            res.json().await.map_err(|e| ModelError::ProviderError {
                provider_id: self.id.clone(),
                message: format!("Failed to parse response: {}", e),
            })?;

        let first_choice = parsed
            .choices
            .first()
            .ok_or_else(|| ModelError::ProviderError {
                provider_id: self.id.clone(),
                message: "Provider returned zero choices".to_string(),
            })?;

        let content = first_choice.message.content.clone().unwrap_or_default();

        let usage = parsed
            .usage
            .map(|u| Usage {
                input_tokens: u.prompt_tokens,
                output_tokens: u.completion_tokens,
                total_tokens: u.total_tokens,
            })
            .unwrap_or_default();

        let finish_reason = match first_choice.finish_reason.as_deref() {
            Some("stop") => FinishReason::Stop,
            Some("length") => FinishReason::Length,
            Some("tool_calls") => FinishReason::ToolCalls,
            Some("content_filter") => FinishReason::ContentFilter,
            _ => FinishReason::Unknown,
        };

        Ok(ModelResponse {
            model_id: request.model_id.clone(),
            content,
            tool_calls: vec![],
            usage,
            finish_reason,
            metadata: HashMap::new(),
        })
    }

    async fn stream(
        &self,
        request: &ModelRequest,
    ) -> Result<mpsc::Receiver<ModelStreamEvent>, ModelError> {
        let (tx, rx) = mpsc::channel(32);
        let resp = self.invoke(request).await?;

        tokio::spawn(async move {
            let text = resp.content.clone();
            let _ = tx.send(ModelStreamEvent::TextDelta(text)).await;
            let _ = tx.send(ModelStreamEvent::Completed(resp)).await;
        });

        Ok(rx)
    }
}

// ============================================================================
// GOOGLE GEMINI ADAPTER
// ============================================================================
pub struct GoogleAdapter {
    id: String,
    name: String,
    config: ProviderConfig,
    http_client: Client,
}

impl GoogleAdapter {
    pub fn new(id: impl Into<String>, name: impl Into<String>, config: ProviderConfig) -> Self {
        let http_client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .unwrap_or_else(|_| Client::new());

        Self {
            id: id.into(),
            name: name.into(),
            config,
            http_client,
        }
    }
}

#[async_trait::async_trait]
impl ModelProvider for GoogleAdapter {
    fn provider_info(&self) -> ProviderDescriptor {
        ProviderDescriptor {
            id: self.id.clone(),
            name: self.name.clone(),
            provider_type: ProviderType::Google,
            execution_mode: ExecutionMode::Cloud,
            enabled: true,
            status: ProviderStatus::Available,
            base_url: self.config.base_url.clone(),
        }
    }

    fn capabilities(&self) -> Vec<ModelCapability> {
        vec![
            ModelCapability::TextGeneration,
            ModelCapability::Vision,
            ModelCapability::ImageInput,
            ModelCapability::CodeGeneration,
            ModelCapability::ToolCalling,
            ModelCapability::StructuredOutput,
        ]
    }

    async fn list_models(&self) -> Vec<ModelDescriptor> {
        vec![
            ModelDescriptor {
                id: format!("{}:gemini-1.5-pro", self.id),
                provider_id: self.id.clone(),
                model_identifier: "gemini-1.5-pro".to_string(),
                display_name: "Google Gemini 1.5 Pro".to_string(),
                execution_mode: ExecutionMode::Cloud,
                capabilities: vec![
                    ModelCapability::TextGeneration,
                    ModelCapability::Vision,
                    ModelCapability::ImageInput,
                    ModelCapability::CodeGeneration,
                    ModelCapability::ToolCalling,
                    ModelCapability::StructuredOutput,
                ],
                context_window: Some(2_000_000),
                max_output_tokens: Some(8192),
                tokenizer: TokenizerInfo::Estimated { factor: 1.0 },
                hardware_requirements: None,
                availability: ModelAvailability::Available,
                metadata: HashMap::new(),
            },
            ModelDescriptor {
                id: format!("{}:gemini-1.5-flash", self.id),
                provider_id: self.id.clone(),
                model_identifier: "gemini-1.5-flash".to_string(),
                display_name: "Google Gemini 1.5 Flash".to_string(),
                execution_mode: ExecutionMode::Cloud,
                capabilities: vec![
                    ModelCapability::TextGeneration,
                    ModelCapability::Vision,
                    ModelCapability::ImageInput,
                    ModelCapability::CodeGeneration,
                    ModelCapability::ToolCalling,
                    ModelCapability::StructuredOutput,
                ],
                context_window: Some(1_000_000),
                max_output_tokens: Some(8192),
                tokenizer: TokenizerInfo::Estimated { factor: 1.0 },
                hardware_requirements: None,
                availability: ModelAvailability::Available,
                metadata: HashMap::new(),
            },
        ]
    }

    async fn health_check(&self) -> ProviderHealth {
        let start = Instant::now();
        let api_key = match &self.config.api_key {
            Some(key) if !key.trim().is_empty() => key.trim(),
            _ => {
                return ProviderHealth {
                    provider_id: self.id.clone(),
                    status: ProviderStatus::Unavailable,
                    latency_ms: start.elapsed().as_millis(),
                    checked_at_timestamp: now_timestamp(),
                    error_message: Some("Missing Gemini API key".to_string()),
                }
            }
        };

        let endpoint = format!(
            "https://generativelanguage.googleapis.com/v1beta/models?key={}",
            api_key
        );

        match self.http_client.get(&endpoint).send().await {
            Ok(resp) => {
                let status = if resp.status().is_success() {
                    ProviderStatus::Available
                } else {
                    ProviderStatus::Unavailable
                };

                ProviderHealth {
                    provider_id: self.id.clone(),
                    status,
                    latency_ms: start.elapsed().as_millis(),
                    checked_at_timestamp: now_timestamp(),
                    error_message: if status == ProviderStatus::Available {
                        None
                    } else {
                        Some(format!("HTTP {}", resp.status()))
                    },
                }
            }
            Err(e) => ProviderHealth {
                provider_id: self.id.clone(),
                status: ProviderStatus::Unavailable,
                latency_ms: start.elapsed().as_millis(),
                checked_at_timestamp: now_timestamp(),
                error_message: Some(e.to_string()),
            },
        }
    }

    async fn invoke(&self, request: &ModelRequest) -> Result<ModelResponse, ModelError> {
        let api_key = self
            .config
            .api_key
            .as_ref()
            .filter(|k| !k.trim().is_empty())
            .ok_or_else(|| ModelError::AuthenticationFailed {
                provider_id: self.id.clone(),
                message: "Missing Gemini API Key".to_string(),
            })?;

        let model_name = request
            .model_id
            .split_once(':')
            .map(|(_, m)| m)
            .unwrap_or(&request.model_id);

        let endpoint = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            model_name, api_key
        );

        let mut parts = Vec::new();
        if let Some(sys) = &request.system_instructions {
            parts.push(serde_json::json!({ "text": format!("System: {}\n", sys) }));
        }

        for msg in &request.messages {
            parts.push(serde_json::json!({ "text": msg.content }));
        }

        let body = serde_json::json!({
            "contents": [{
                "parts": parts
            }]
        });

        let resp = self
            .http_client
            .post(&endpoint)
            .json(&body)
            .send()
            .await
            .map_err(|e| ModelError::NetworkError {
                message: e.to_string(),
            })?;

        if !resp.status().is_success() {
            let err_text = resp.text().await.unwrap_or_default();
            return Err(ModelError::ProviderError {
                provider_id: self.id.clone(),
                message: format!("Gemini API error: {}", err_text),
            });
        }

        #[derive(Deserialize)]
        struct Part {
            text: Option<String>,
        }
        #[derive(Deserialize)]
        struct Content {
            parts: Vec<Part>,
        }
        #[derive(Deserialize)]
        struct Candidate {
            content: Content,
        }
        #[derive(Deserialize)]
        struct GeminiResponse {
            candidates: Option<Vec<Candidate>>,
        }

        let parsed: GeminiResponse = resp.json().await.map_err(|e| ModelError::ProviderError {
            provider_id: self.id.clone(),
            message: format!("Failed to parse Gemini response: {}", e),
        })?;

        let text = parsed
            .candidates
            .as_ref()
            .and_then(|c| c.first())
            .and_then(|c| c.content.parts.first())
            .and_then(|p| p.text.clone())
            .unwrap_or_default();

        Ok(ModelResponse {
            model_id: request.model_id.clone(),
            content: text,
            tool_calls: vec![],
            usage: Usage::default(),
            finish_reason: FinishReason::Stop,
            metadata: HashMap::new(),
        })
    }

    async fn stream(
        &self,
        request: &ModelRequest,
    ) -> Result<mpsc::Receiver<ModelStreamEvent>, ModelError> {
        let (tx, rx) = mpsc::channel(32);
        let resp = self.invoke(request).await?;

        tokio::spawn(async move {
            let text = resp.content.clone();
            let _ = tx.send(ModelStreamEvent::TextDelta(text)).await;
            let _ = tx.send(ModelStreamEvent::Completed(resp)).await;
        });

        Ok(rx)
    }
}

// ============================================================================
// OLLAMA (LOCAL) ADAPTER
// ============================================================================
pub struct OllamaAdapter {
    id: String,
    name: String,
    config: ProviderConfig,
    http_client: Client,
}

impl OllamaAdapter {
    pub fn new(id: impl Into<String>, name: impl Into<String>, config: ProviderConfig) -> Self {
        let http_client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .unwrap_or_else(|_| Client::new());

        Self {
            id: id.into(),
            name: name.into(),
            config,
            http_client,
        }
    }
}

#[async_trait::async_trait]
impl ModelProvider for OllamaAdapter {
    fn provider_info(&self) -> ProviderDescriptor {
        ProviderDescriptor {
            id: self.id.clone(),
            name: self.name.clone(),
            provider_type: ProviderType::Ollama,
            execution_mode: ExecutionMode::Local,
            enabled: true,
            status: ProviderStatus::Available,
            base_url: self.config.base_url.clone(),
        }
    }

    fn capabilities(&self) -> Vec<ModelCapability> {
        vec![
            ModelCapability::TextGeneration,
            ModelCapability::CodeGeneration,
            ModelCapability::Streaming,
        ]
    }

    async fn list_models(&self) -> Vec<ModelDescriptor> {
        vec![ModelDescriptor {
            id: format!("{}:{}", self.id, self.config.default_model),
            provider_id: self.id.clone(),
            model_identifier: self.config.default_model.clone(),
            display_name: format!("Local Ollama ({})", self.config.default_model),
            execution_mode: ExecutionMode::Local,
            capabilities: self.capabilities(),
            context_window: Some(8192),
            max_output_tokens: Some(4096),
            tokenizer: TokenizerInfo::Estimated { factor: 1.0 },
            hardware_requirements: Some(HardwareRequirement {
                minimum_ram: Some(8 * 1024 * 1024 * 1024),
                recommended_ram: Some(16 * 1024 * 1024 * 1024),
                minimum_vram: Some(4 * 1024 * 1024 * 1024),
                recommended_vram: Some(8 * 1024 * 1024 * 1024),
            }),
            availability: ModelAvailability::Available,
            metadata: HashMap::new(),
        }]
    }

    async fn health_check(&self) -> ProviderHealth {
        let start = Instant::now();
        let base_url = self
            .config
            .base_url
            .as_deref()
            .unwrap_or("http://127.0.0.1:11434");
        let endpoint = format!("{}/api/tags", base_url.trim_end_matches('/'));

        match self.http_client.get(&endpoint).send().await {
            Ok(resp) => {
                let status = if resp.status().is_success() {
                    ProviderStatus::Available
                } else {
                    ProviderStatus::Unavailable
                };

                ProviderHealth {
                    provider_id: self.id.clone(),
                    status,
                    latency_ms: start.elapsed().as_millis(),
                    checked_at_timestamp: now_timestamp(),
                    error_message: if status == ProviderStatus::Available {
                        None
                    } else {
                        Some(format!("HTTP {}", resp.status()))
                    },
                }
            }
            Err(e) => ProviderHealth {
                provider_id: self.id.clone(),
                status: ProviderStatus::Unavailable,
                latency_ms: start.elapsed().as_millis(),
                checked_at_timestamp: now_timestamp(),
                error_message: Some(e.to_string()),
            },
        }
    }

    async fn invoke(&self, request: &ModelRequest) -> Result<ModelResponse, ModelError> {
        let base_url = self
            .config
            .base_url
            .as_deref()
            .unwrap_or("http://127.0.0.1:11434");
        let endpoint = format!("{}/api/generate", base_url.trim_end_matches('/'));

        let target_model = request
            .model_id
            .split_once(':')
            .map(|(_, m)| m)
            .unwrap_or(&request.model_id);

        let prompt_text = request
            .messages
            .iter()
            .map(|m| m.content.as_str())
            .collect::<Vec<_>>()
            .join("\n");

        let body = serde_json::json!({
            "model": target_model,
            "prompt": prompt_text,
            "stream": false
        });

        let resp = self
            .http_client
            .post(&endpoint)
            .json(&body)
            .send()
            .await
            .map_err(|e| ModelError::NetworkError {
                message: e.to_string(),
            })?;

        if !resp.status().is_success() {
            let err_text = resp.text().await.unwrap_or_default();
            return Err(ModelError::ProviderError {
                provider_id: self.id.clone(),
                message: format!("Ollama API Error: {}", err_text),
            });
        }

        #[derive(Deserialize)]
        struct OllamaGenResponse {
            response: String,
        }

        let parsed: OllamaGenResponse =
            resp.json().await.map_err(|e| ModelError::ProviderError {
                provider_id: self.id.clone(),
                message: format!("Failed to parse Ollama response: {}", e),
            })?;

        Ok(ModelResponse {
            model_id: request.model_id.clone(),
            content: parsed.response,
            tool_calls: vec![],
            usage: Usage::default(),
            finish_reason: FinishReason::Stop,
            metadata: HashMap::new(),
        })
    }

    async fn stream(
        &self,
        request: &ModelRequest,
    ) -> Result<mpsc::Receiver<ModelStreamEvent>, ModelError> {
        let (tx, rx) = mpsc::channel(32);
        let resp = self.invoke(request).await?;

        tokio::spawn(async move {
            let text = resp.content.clone();
            let _ = tx.send(ModelStreamEvent::TextDelta(text)).await;
            let _ = tx.send(ModelStreamEvent::Completed(resp)).await;
        });

        Ok(rx)
    }
}

// ============================================================================
// MOCK ADAPTER FOR TESTING & SIMULATION
// ============================================================================
#[cfg(test)]
pub struct MockAdapter {
    pub descriptor: ProviderDescriptor,
    pub mock_models: Vec<ModelDescriptor>,
    pub should_fail: bool,
    pub fail_error: Option<ModelError>,
    pub mock_response: Option<String>,
}

#[cfg(test)]
impl MockAdapter {
    pub fn new(id: &str, execution_mode: ExecutionMode) -> Self {
        let descriptor = ProviderDescriptor {
            id: id.to_string(),
            name: format!("Mock Provider ({})", id),
            provider_type: ProviderType::Custom,
            execution_mode,
            enabled: true,
            status: ProviderStatus::Available,
            base_url: Some("http://mock.local".to_string()),
        };

        let mock_model = ModelDescriptor {
            id: format!("{}:mock-model", id),
            provider_id: id.to_string(),
            model_identifier: "mock-model".to_string(),
            display_name: "Mock Model 1.0".to_string(),
            execution_mode,
            capabilities: vec![
                ModelCapability::TextGeneration,
                ModelCapability::CodeGeneration,
                ModelCapability::ToolCalling,
                ModelCapability::Streaming,
            ],
            context_window: Some(16384),
            max_output_tokens: Some(2048),
            tokenizer: TokenizerInfo::Exact {
                name: "mock-tokenizer".to_string(),
            },
            hardware_requirements: None,
            availability: ModelAvailability::Available,
            metadata: HashMap::new(),
        };

        Self {
            descriptor,
            mock_models: vec![mock_model],
            should_fail: false,
            fail_error: None,
            mock_response: None,
        }
    }
}

#[cfg(test)]
#[async_trait::async_trait]
impl ModelProvider for MockAdapter {
    fn provider_info(&self) -> ProviderDescriptor {
        self.descriptor.clone()
    }

    fn capabilities(&self) -> Vec<ModelCapability> {
        vec![
            ModelCapability::TextGeneration,
            ModelCapability::CodeGeneration,
            ModelCapability::ToolCalling,
            ModelCapability::Streaming,
        ]
    }

    async fn list_models(&self) -> Vec<ModelDescriptor> {
        self.mock_models.clone()
    }

    async fn health_check(&self) -> ProviderHealth {
        ProviderHealth {
            provider_id: self.descriptor.id.clone(),
            status: if self.should_fail {
                ProviderStatus::Unavailable
            } else {
                ProviderStatus::Available
            },
            latency_ms: 5,
            checked_at_timestamp: now_timestamp(),
            error_message: if self.should_fail {
                Some("Mock provider error".to_string())
            } else {
                None
            },
        }
    }

    async fn invoke(&self, request: &ModelRequest) -> Result<ModelResponse, ModelError> {
        if self.should_fail {
            if let Some(err) = &self.fail_error {
                return Err(err.clone());
            }
            return Err(ModelError::ProviderUnavailable {
                provider_id: self.descriptor.id.clone(),
                reason: "Simulated mock provider failure".to_string(),
            });
        }

        let content = self.mock_response.clone().unwrap_or_else(|| {
            format!(
                "Mock response to: {}",
                request
                    .messages
                    .last()
                    .map(|m| m.content.as_str())
                    .unwrap_or("")
            )
        });

        Ok(ModelResponse {
            model_id: request.model_id.clone(),
            content,
            tool_calls: vec![],
            usage: Usage {
                input_tokens: Some(10),
                output_tokens: Some(20),
                total_tokens: Some(30),
            },
            finish_reason: FinishReason::Stop,
            metadata: HashMap::new(),
        })
    }

    async fn stream(
        &self,
        request: &ModelRequest,
    ) -> Result<mpsc::Receiver<ModelStreamEvent>, ModelError> {
        if self.should_fail {
            return Err(ModelError::StreamFailed {
                message: "Mock stream failure".to_string(),
            });
        }

        let (tx, rx) = mpsc::channel(10);
        let resp = self.invoke(request).await?;

        tokio::spawn(async move {
            let _ = tx
                .send(ModelStreamEvent::TextDelta(resp.content.clone()))
                .await;
            let _ = tx.send(ModelStreamEvent::Completed(resp)).await;
        });

        Ok(rx)
    }
}
