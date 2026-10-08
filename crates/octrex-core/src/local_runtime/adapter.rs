use async_trait::async_trait;
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;

use crate::config::ProviderType;
use crate::local_runtime::errors::LocalRuntimeError;
use crate::local_runtime::types::{
    now_timestamp, LocalExecutionMode, LocalModelState, LocalRuntimeDescriptor, LocalRuntimeHealth,
    LocalRuntimeHealthReport, LocalRuntimeType,
};
use crate::models::{
    ModelAvailability, ModelCapability, ModelDescriptor, ModelError, ModelRequest,
};

// ============================================================================
// DISCOVERED MODEL (adapter-neutral)
// ============================================================================

/// Normalized model facts reported by a runtime adapter.
///
/// All resource fields are optional: unknown stays unknown.
#[derive(Debug, Clone)]
pub struct DiscoveredModel {
    pub model_identifier: String,
    pub display_name: String,
    pub context_window: Option<u32>,
    pub max_output_tokens: Option<u32>,
    pub capabilities: Vec<ModelCapability>,
    pub quantization: Option<String>,
    pub parameter_count_billions: Option<f64>,
    pub architecture: Option<String>,
    pub model_format: Option<String>,
    pub required_ram_mb: Option<u64>,
    pub required_vram_mb: Option<u64>,
}

impl DiscoveredModel {
    pub fn basic(identifier: impl Into<String>) -> Self {
        let id = identifier.into();
        Self {
            display_name: id.clone(),
            model_identifier: id,
            context_window: None,
            max_output_tokens: None,
            capabilities: vec![ModelCapability::TextGeneration, ModelCapability::Streaming],
            quantization: None,
            parameter_count_billions: None,
            architecture: None,
            model_format: None,
            required_ram_mb: None,
            required_vram_mb: None,
        }
    }
}

// ============================================================================
// ADAPTER TRAIT
// ============================================================================

/// Provider-neutral interface every local runtime must implement.
///
/// Adapters never execute binaries and never touch the network beyond their
/// own validated loopback endpoint.
#[async_trait]
pub trait LocalRuntimeAdapter: Send + Sync {
    fn runtime_type(&self) -> LocalRuntimeType;
    fn endpoint(&self) -> &str;

    async fn probe_health(&self) -> LocalRuntimeHealthReport;
    async fn list_models(&self) -> Result<Vec<DiscoveredModel>, LocalRuntimeError>;
    async fn invoke(
        &self,
        model_identifier: &str,
        request: &ModelRequest,
        timeout: Duration,
    ) -> Result<crate::models::ModelResponse, LocalRuntimeError>;
    async fn stream(
        &self,
        model_identifier: &str,
        request: &ModelRequest,
        timeout: Duration,
    ) -> Result<mpsc::Receiver<crate::models::ModelStreamEvent>, LocalRuntimeError>;
}

fn http_client(timeout: Duration) -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(timeout)
        .build()
        .unwrap_or_else(|_| reqwest::Client::new())
}

fn base_of(endpoint: &str) -> String {
    endpoint.trim_end_matches('/').to_string()
}

fn prompt_of(request: &ModelRequest) -> String {
    let mut parts = Vec::new();
    if let Some(sys) = &request.system_instructions {
        parts.push(format!("System: {}", sys));
    }
    for m in &request.messages {
        parts.push(format!("{}: {}", m.role, m.content));
    }
    parts.join("\n")
}

// ============================================================================
// OLLAMA ADAPTER (native /api/tags + /api/generate + /api/show)
// ============================================================================

pub struct OllamaRuntimeAdapter {
    runtime_id: String,
    endpoint: String,
}

impl OllamaRuntimeAdapter {
    pub fn new(runtime_id: impl Into<String>, endpoint: impl Into<String>) -> Self {
        Self {
            runtime_id: runtime_id.into(),
            endpoint: endpoint.into(),
        }
    }
}

#[derive(Debug, Deserialize)]
struct OllamaTagsResponse {
    models: Option<Vec<OllamaTagModel>>,
}

#[derive(Debug, Deserialize)]
struct OllamaTagModel {
    name: Option<String>,
    model: Option<String>,
    details: Option<OllamaModelDetails>,
}

#[derive(Debug, Deserialize)]
struct OllamaModelDetails {
    parameter_size: Option<String>,
    quantization_level: Option<String>,
    family: Option<String>,
    format: Option<String>,
}

fn parse_param_size(raw: Option<&str>) -> Option<f64> {
    let s = raw?.to_lowercase().replace([' ', '_'], "");
    let s = s.trim_end_matches('b');
    // Handles "7b", "7.2b", "70b", "0.5b".
    s.parse::<f64>().ok().filter(|v| *v > 0.0)
}

#[async_trait]
impl LocalRuntimeAdapter for OllamaRuntimeAdapter {
    fn runtime_type(&self) -> LocalRuntimeType {
        LocalRuntimeType::Ollama
    }

    fn endpoint(&self) -> &str {
        &self.endpoint
    }

    async fn probe_health(&self) -> LocalRuntimeHealthReport {
        let start = Instant::now();
        let url = format!("{}/api/tags", base_of(&self.endpoint));
        let client = http_client(Duration::from_secs(5));
        match client.get(&url).send().await {
            Ok(resp) if resp.status().is_success() => {
                let count = resp
                    .json::<OllamaTagsResponse>()
                    .await
                    .map(|t| t.models.map(|m| m.len()).unwrap_or(0))
                    .unwrap_or(0);
                LocalRuntimeHealthReport {
                    runtime_id: self.runtime_id.clone(),
                    health: LocalRuntimeHealth::Healthy,
                    reachable: true,
                    models_available: count,
                    models_loaded: 0,
                    latency_ms: Some(start.elapsed().as_millis()),
                    last_success_timestamp: Some(now_timestamp()),
                    last_error: None,
                    checked_at_timestamp: now_timestamp(),
                }
            }
            Ok(resp) => LocalRuntimeHealthReport {
                runtime_id: self.runtime_id.clone(),
                health: LocalRuntimeHealth::Degraded,
                reachable: true,
                models_available: 0,
                models_loaded: 0,
                latency_ms: Some(start.elapsed().as_millis()),
                last_success_timestamp: None,
                last_error: Some(format!("HTTP {}", resp.status())),
                checked_at_timestamp: now_timestamp(),
            },
            Err(e) => LocalRuntimeHealthReport {
                runtime_id: self.runtime_id.clone(),
                health: LocalRuntimeHealth::Unavailable,
                reachable: false,
                models_available: 0,
                models_loaded: 0,
                latency_ms: Some(start.elapsed().as_millis()),
                last_success_timestamp: None,
                last_error: Some(sanitize_reqwest_error(&e)),
                checked_at_timestamp: now_timestamp(),
            },
        }
    }

    async fn list_models(&self) -> Result<Vec<DiscoveredModel>, LocalRuntimeError> {
        let url = format!("{}/api/tags", base_of(&self.endpoint));
        let client = http_client(Duration::from_secs(10));
        let resp =
            client
                .get(&url)
                .send()
                .await
                .map_err(|e| LocalRuntimeError::RuntimeUnavailable {
                    reason: format!("Ollama tags request failed: {}", sanitize_reqwest_error(&e)),
                })?;
        if !resp.status().is_success() {
            return Err(LocalRuntimeError::RuntimeUnavailable {
                reason: format!("Ollama tags returned HTTP {}", resp.status()),
            });
        }
        let parsed: OllamaTagsResponse =
            resp.json()
                .await
                .map_err(|e| LocalRuntimeError::MalformedResponse {
                    reason: format!("Failed to parse Ollama tags: {}", e),
                })?;
        let models = parsed.models.unwrap_or_default();
        Ok(models
            .into_iter()
            .map(|m| {
                let identifier = m.model.or(m.name).unwrap_or_else(|| "unknown".to_string());
                let mut d = DiscoveredModel::basic(identifier.clone());
                d.display_name = format!("Ollama {}", identifier);
                if let Some(details) = m.details {
                    d.parameter_count_billions =
                        parse_param_size(details.parameter_size.as_deref());
                    d.quantization = details.quantization_level;
                    d.architecture = details.family;
                    d.model_format = details.format.or(Some("ollama".to_string()));
                } else {
                    d.model_format = Some("ollama".to_string());
                }
                d.capabilities = vec![
                    ModelCapability::TextGeneration,
                    ModelCapability::CodeGeneration,
                    ModelCapability::Streaming,
                ];
                d
            })
            .collect())
    }

    async fn invoke(
        &self,
        model_identifier: &str,
        request: &ModelRequest,
        timeout: Duration,
    ) -> Result<crate::models::ModelResponse, LocalRuntimeError> {
        let url = format!("{}/api/generate", base_of(&self.endpoint));
        let client = http_client(timeout);
        let body = serde_json::json!({
            "model": model_identifier,
            "prompt": prompt_of(request),
            "stream": false,
        });
        let resp = client.post(&url).json(&body).send().await.map_err(|e| {
            if e.is_timeout() {
                LocalRuntimeError::Timeout {
                    timeout_ms: timeout.as_millis() as u64,
                }
            } else {
                LocalRuntimeError::RuntimeUnavailable {
                    reason: format!("Ollama generate failed: {}", sanitize_reqwest_error(&e)),
                }
            }
        })?;
        if !resp.status().is_success() {
            return Err(LocalRuntimeError::RuntimeUnavailable {
                reason: format!("Ollama generate returned HTTP {}", resp.status()),
            });
        }
        #[derive(Deserialize)]
        struct GenResponse {
            response: Option<String>,
        }
        let parsed: GenResponse =
            resp.json()
                .await
                .map_err(|e| LocalRuntimeError::MalformedResponse {
                    reason: format!("Failed to parse Ollama response: {}", e),
                })?;
        Ok(crate::models::ModelResponse {
            model_id: request.model_id.clone(),
            content: parsed.response.unwrap_or_default(),
            tool_calls: vec![],
            usage: crate::models::Usage::default(),
            finish_reason: crate::models::FinishReason::Stop,
            metadata: HashMap::new(),
        })
    }

    async fn stream(
        &self,
        model_identifier: &str,
        request: &ModelRequest,
        timeout: Duration,
    ) -> Result<mpsc::Receiver<crate::models::ModelStreamEvent>, LocalRuntimeError> {
        use futures_util::StreamExt;
        let url = format!("{}/api/generate", base_of(&self.endpoint));
        let client = http_client(timeout);
        let body = serde_json::json!({
            "model": model_identifier,
            "prompt": prompt_of(request),
            "stream": true,
        });
        let resp = client.post(&url).json(&body).send().await.map_err(|e| {
            if e.is_timeout() {
                LocalRuntimeError::Timeout {
                    timeout_ms: timeout.as_millis() as u64,
                }
            } else {
                LocalRuntimeError::RuntimeUnavailable {
                    reason: format!("Ollama stream failed: {}", sanitize_reqwest_error(&e)),
                }
            }
        })?;
        if !resp.status().is_success() {
            return Err(LocalRuntimeError::RuntimeUnavailable {
                reason: format!("Ollama stream returned HTTP {}", resp.status()),
            });
        }
        let (tx, rx) = mpsc::channel(64);
        let model_id = request.model_id.clone();
        tokio::spawn(async move {
            let mut stream = resp.bytes_stream();
            let mut buffer = Vec::new();
            let mut content = String::new();
            while let Some(chunk) = stream.next().await {
                match chunk {
                    Ok(bytes) => {
                        buffer.extend_from_slice(&bytes);
                        // Ollama streams NDJSON: split on newlines.
                        while let Some(pos) = buffer.iter().position(|b| *b == b'\n') {
                            let line: Vec<u8> = buffer.drain(..=pos).collect();
                            let text = String::from_utf8_lossy(&line);
                            let trimmed = text.trim();
                            if trimmed.is_empty() {
                                continue;
                            }
                            if let Ok(v) = serde_json::from_str::<serde_json::Value>(trimmed) {
                                if let Some(delta) = v.get("response").and_then(|r| r.as_str()) {
                                    if !delta.is_empty() {
                                        content.push_str(delta);
                                        if tx
                                            .send(crate::models::ModelStreamEvent::TextDelta(
                                                delta.to_string(),
                                            ))
                                            .await
                                            .is_err()
                                        {
                                            return;
                                        }
                                    }
                                }
                                if v.get("done").and_then(|d| d.as_bool()).unwrap_or(false) {
                                    break;
                                }
                            }
                        }
                    }
                    Err(e) => {
                        let _ = tx
                            .send(crate::models::ModelStreamEvent::Failed(format!(
                                "Ollama stream error: {}",
                                e
                            )))
                            .await;
                        return;
                    }
                }
            }
            let done = crate::models::ModelResponse {
                model_id,
                content,
                tool_calls: vec![],
                usage: crate::models::Usage::default(),
                finish_reason: crate::models::FinishReason::Stop,
                metadata: HashMap::new(),
            };
            let _ = tx
                .send(crate::models::ModelStreamEvent::Completed(done))
                .await;
        });
        Ok(rx)
    }
}

pub fn sanitize_reqwest_error(e: &reqwest::Error) -> String {
    // Never leak internal paths or credentials: reqwest errors may embed URLs.
    let msg = e.to_string();
    if msg.contains('@') {
        "HTTP request failed (credentials redacted)".to_string()
    } else {
        // Strip any query string that might carry secrets.
        match msg.find('?') {
            Some(idx) => format!("{}<query redacted>", &msg[..idx]),
            None => msg,
        }
    }
}

// ============================================================================
// LLAMA.CPP SERVER ADAPTER (/v1/models + /v1/chat/completions, OpenAI-style)
// ============================================================================

pub struct LlamaCppRuntimeAdapter {
    runtime_id: String,
    endpoint: String,
}

impl LlamaCppRuntimeAdapter {
    pub fn new(runtime_id: impl Into<String>, endpoint: impl Into<String>) -> Self {
        Self {
            runtime_id: runtime_id.into(),
            endpoint: endpoint.into(),
        }
    }
}

#[derive(Debug, Deserialize)]
struct OpenAiModelList {
    data: Option<Vec<OpenAiModelEntry>>,
}

#[derive(Debug, Deserialize)]
struct OpenAiModelEntry {
    id: Option<String>,
}

async fn probe_openai_style(
    runtime_id: &str,
    endpoint: &str,
    label: &str,
) -> LocalRuntimeHealthReport {
    let start = Instant::now();
    let url = format!("{}/v1/models", base_of(endpoint));
    let client = http_client(Duration::from_secs(5));
    match client.get(&url).send().await {
        Ok(resp) if resp.status().is_success() => {
            let count = resp
                .json::<OpenAiModelList>()
                .await
                .map(|l| l.data.map(|d| d.len()).unwrap_or(0))
                .unwrap_or(0);
            LocalRuntimeHealthReport {
                runtime_id: runtime_id.to_string(),
                health: LocalRuntimeHealth::Healthy,
                reachable: true,
                models_available: count,
                models_loaded: 0,
                latency_ms: Some(start.elapsed().as_millis()),
                last_success_timestamp: Some(now_timestamp()),
                last_error: None,
                checked_at_timestamp: now_timestamp(),
            }
        }
        Ok(resp) => LocalRuntimeHealthReport {
            runtime_id: runtime_id.to_string(),
            health: LocalRuntimeHealth::Degraded,
            reachable: true,
            models_available: 0,
            models_loaded: 0,
            latency_ms: Some(start.elapsed().as_millis()),
            last_success_timestamp: None,
            last_error: Some(format!("{} models returned HTTP {}", label, resp.status())),
            checked_at_timestamp: now_timestamp(),
        },
        Err(e) => LocalRuntimeHealthReport {
            runtime_id: runtime_id.to_string(),
            health: LocalRuntimeHealth::Unavailable,
            reachable: false,
            models_available: 0,
            models_loaded: 0,
            latency_ms: Some(start.elapsed().as_millis()),
            last_success_timestamp: None,
            last_error: Some(sanitize_reqwest_error(&e)),
            checked_at_timestamp: now_timestamp(),
        },
    }
}

async fn list_openai_style_models(
    endpoint: &str,
    label: &str,
) -> Result<Vec<DiscoveredModel>, LocalRuntimeError> {
    let url = format!("{}/v1/models", base_of(endpoint));
    let client = http_client(Duration::from_secs(10));
    let resp =
        client
            .get(&url)
            .send()
            .await
            .map_err(|e| LocalRuntimeError::RuntimeUnavailable {
                reason: format!(
                    "{} models request failed: {}",
                    label,
                    sanitize_reqwest_error(&e)
                ),
            })?;
    if !resp.status().is_success() {
        return Err(LocalRuntimeError::RuntimeUnavailable {
            reason: format!("{} models returned HTTP {}", label, resp.status()),
        });
    }
    let parsed: OpenAiModelList =
        resp.json()
            .await
            .map_err(|e| LocalRuntimeError::MalformedResponse {
                reason: format!("Failed to parse {} models: {}", label, e),
            })?;
    Ok(parsed
        .data
        .unwrap_or_default()
        .into_iter()
        .map(|m| {
            let id = m.id.unwrap_or_else(|| "default".to_string());
            let mut d = DiscoveredModel::basic(id.clone());
            d.display_name = format!("{} {}", label, id);
            d.model_format = Some("openai-compatible".to_string());
            d.capabilities = vec![ModelCapability::TextGeneration, ModelCapability::Streaming];
            d
        })
        .collect())
}

async fn invoke_openai_chat(
    endpoint: &str,
    model_identifier: &str,
    request: &ModelRequest,
    timeout: Duration,
    label: &str,
) -> Result<crate::models::ModelResponse, LocalRuntimeError> {
    let url = format!("{}/v1/chat/completions", base_of(endpoint));
    let client = http_client(timeout);
    let mut messages = Vec::new();
    if let Some(sys) = &request.system_instructions {
        messages.push(serde_json::json!({"role": "system", "content": sys}));
    }
    for m in &request.messages {
        messages.push(serde_json::json!({"role": m.role, "content": m.content}));
    }
    let mut body = serde_json::json!({
        "model": model_identifier,
        "messages": messages,
        "temperature": request.temperature.unwrap_or(0.3),
        "stream": false,
    });
    if let Some(max_tokens) = request.max_output_tokens {
        body["max_tokens"] = serde_json::json!(max_tokens);
    }
    let resp = client.post(&url).json(&body).send().await.map_err(|e| {
        if e.is_timeout() {
            LocalRuntimeError::Timeout {
                timeout_ms: timeout.as_millis() as u64,
            }
        } else {
            LocalRuntimeError::RuntimeUnavailable {
                reason: format!(
                    "{} completions failed: {}",
                    label,
                    sanitize_reqwest_error(&e)
                ),
            }
        }
    })?;
    if !resp.status().is_success() {
        return Err(LocalRuntimeError::RuntimeUnavailable {
            reason: format!("{} completions returned HTTP {}", label, resp.status()),
        });
    }
    #[derive(Deserialize)]
    struct ChoiceMessage {
        content: Option<String>,
    }
    #[derive(Deserialize)]
    struct Choice {
        message: Option<ChoiceMessage>,
    }
    #[derive(Deserialize)]
    struct ChatResponse {
        choices: Option<Vec<Choice>>,
    }
    let parsed: ChatResponse =
        resp.json()
            .await
            .map_err(|e| LocalRuntimeError::MalformedResponse {
                reason: format!("Failed to parse {} response: {}", label, e),
            })?;
    let content = parsed
        .choices
        .and_then(|c| c.into_iter().next())
        .and_then(|c| c.message)
        .and_then(|m| m.content)
        .unwrap_or_default();
    Ok(crate::models::ModelResponse {
        model_id: request.model_id.clone(),
        content,
        tool_calls: vec![],
        usage: crate::models::Usage::default(),
        finish_reason: crate::models::FinishReason::Stop,
        metadata: HashMap::new(),
    })
}

async fn stream_openai_chat(
    endpoint: &str,
    model_identifier: &str,
    request: &ModelRequest,
    timeout: Duration,
    label: &str,
) -> Result<mpsc::Receiver<crate::models::ModelStreamEvent>, LocalRuntimeError> {
    use futures_util::StreamExt;
    let url = format!("{}/v1/chat/completions", base_of(endpoint));
    let client = http_client(timeout);
    let mut messages = Vec::new();
    if let Some(sys) = &request.system_instructions {
        messages.push(serde_json::json!({"role": "system", "content": sys}));
    }
    for m in &request.messages {
        messages.push(serde_json::json!({"role": m.role, "content": m.content}));
    }
    let body = serde_json::json!({
        "model": model_identifier,
        "messages": messages,
        "temperature": request.temperature.unwrap_or(0.3),
        "stream": true,
    });
    let resp = client.post(&url).json(&body).send().await.map_err(|e| {
        if e.is_timeout() {
            LocalRuntimeError::Timeout {
                timeout_ms: timeout.as_millis() as u64,
            }
        } else {
            LocalRuntimeError::RuntimeUnavailable {
                reason: format!("{} stream failed: {}", label, sanitize_reqwest_error(&e)),
            }
        }
    })?;
    if !resp.status().is_success() {
        return Err(LocalRuntimeError::RuntimeUnavailable {
            reason: format!("{} stream returned HTTP {}", label, resp.status()),
        });
    }
    let (tx, rx) = mpsc::channel(64);
    let model_id = request.model_id.clone();
    tokio::spawn(async move {
        let mut stream = resp.bytes_stream();
        let mut buffer = Vec::new();
        let mut content = String::new();
        while let Some(chunk) = stream.next().await {
            match chunk {
                Ok(bytes) => {
                    buffer.extend_from_slice(&bytes);
                    while let Some(pos) = buffer.iter().position(|b| *b == b'\n') {
                        let line: Vec<u8> = buffer.drain(..=pos).collect();
                        let text = String::from_utf8_lossy(&line);
                        let trimmed = text.trim();
                        if trimmed.is_empty() || trimmed == "data: [DONE]" || trimmed == "[DONE]" {
                            if trimmed == "data: [DONE]" || trimmed == "[DONE]" {
                                break;
                            }
                            continue;
                        }
                        let payload = trimmed
                            .strip_prefix("data:")
                            .map(|s| s.trim())
                            .unwrap_or(trimmed);
                        if payload == "[DONE]" {
                            break;
                        }
                        if let Ok(v) = serde_json::from_str::<serde_json::Value>(payload) {
                            let delta = v
                                .get("choices")
                                .and_then(|c| c.get(0))
                                .and_then(|c| c.get("delta"))
                                .and_then(|d| d.get("content"))
                                .and_then(|c| c.as_str())
                                .unwrap_or("");
                            if !delta.is_empty() {
                                content.push_str(delta);
                                if tx
                                    .send(crate::models::ModelStreamEvent::TextDelta(
                                        delta.to_string(),
                                    ))
                                    .await
                                    .is_err()
                                {
                                    return;
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    let _ = tx
                        .send(crate::models::ModelStreamEvent::Failed(format!(
                            "Local stream error: {}",
                            e
                        )))
                        .await;
                    return;
                }
            }
        }
        let done = crate::models::ModelResponse {
            model_id,
            content,
            tool_calls: vec![],
            usage: crate::models::Usage::default(),
            finish_reason: crate::models::FinishReason::Stop,
            metadata: HashMap::new(),
        };
        let _ = tx
            .send(crate::models::ModelStreamEvent::Completed(done))
            .await;
    });
    Ok(rx)
}

#[async_trait]
impl LocalRuntimeAdapter for LlamaCppRuntimeAdapter {
    fn runtime_type(&self) -> LocalRuntimeType {
        LocalRuntimeType::LlamaCppServer
    }

    fn endpoint(&self) -> &str {
        &self.endpoint
    }

    async fn probe_health(&self) -> LocalRuntimeHealthReport {
        probe_openai_style(&self.runtime_id, &self.endpoint, "llama.cpp").await
    }

    async fn list_models(&self) -> Result<Vec<DiscoveredModel>, LocalRuntimeError> {
        list_openai_style_models(&self.endpoint, "llama.cpp").await
    }

    async fn invoke(
        &self,
        model_identifier: &str,
        request: &ModelRequest,
        timeout: Duration,
    ) -> Result<crate::models::ModelResponse, LocalRuntimeError> {
        invoke_openai_chat(
            &self.endpoint,
            model_identifier,
            request,
            timeout,
            "llama.cpp",
        )
        .await
    }

    async fn stream(
        &self,
        model_identifier: &str,
        request: &ModelRequest,
        timeout: Duration,
    ) -> Result<mpsc::Receiver<crate::models::ModelStreamEvent>, LocalRuntimeError> {
        stream_openai_chat(
            &self.endpoint,
            model_identifier,
            request,
            timeout,
            "llama.cpp",
        )
        .await
    }
}

// ============================================================================
// GENERIC LOCAL OPENAI-COMPATIBLE ADAPTER
// ============================================================================

pub struct GenericLocalOpenAiAdapter {
    runtime_id: String,
    endpoint: String,
}

impl GenericLocalOpenAiAdapter {
    pub fn new(runtime_id: impl Into<String>, endpoint: impl Into<String>) -> Self {
        Self {
            runtime_id: runtime_id.into(),
            endpoint: endpoint.into(),
        }
    }
}

#[async_trait]
impl LocalRuntimeAdapter for GenericLocalOpenAiAdapter {
    fn runtime_type(&self) -> LocalRuntimeType {
        LocalRuntimeType::LocalOpenAiCompatible
    }

    fn endpoint(&self) -> &str {
        &self.endpoint
    }

    async fn probe_health(&self) -> LocalRuntimeHealthReport {
        probe_openai_style(&self.runtime_id, &self.endpoint, "local-openai").await
    }

    async fn list_models(&self) -> Result<Vec<DiscoveredModel>, LocalRuntimeError> {
        list_openai_style_models(&self.endpoint, "local-openai").await
    }

    async fn invoke(
        &self,
        model_identifier: &str,
        request: &ModelRequest,
        timeout: Duration,
    ) -> Result<crate::models::ModelResponse, LocalRuntimeError> {
        invoke_openai_chat(
            &self.endpoint,
            model_identifier,
            request,
            timeout,
            "local-openai",
        )
        .await
    }

    async fn stream(
        &self,
        model_identifier: &str,
        request: &ModelRequest,
        timeout: Duration,
    ) -> Result<mpsc::Receiver<crate::models::ModelStreamEvent>, LocalRuntimeError> {
        stream_openai_chat(
            &self.endpoint,
            model_identifier,
            request,
            timeout,
            "local-openai",
        )
        .await
    }
}

/// Build the adapter for a runtime descriptor. Pure function — no I/O.
pub fn adapter_for_descriptor(desc: &LocalRuntimeDescriptor) -> Arc<dyn LocalRuntimeAdapter> {
    match desc.runtime_type {
        LocalRuntimeType::Ollama => Arc::new(OllamaRuntimeAdapter::new(
            desc.id.clone(),
            desc.endpoint.clone(),
        )),
        LocalRuntimeType::LlamaCppServer => Arc::new(LlamaCppRuntimeAdapter::new(
            desc.id.clone(),
            desc.endpoint.clone(),
        )),
        LocalRuntimeType::LocalOpenAiCompatible | LocalRuntimeType::Other => Arc::new(
            GenericLocalOpenAiAdapter::new(desc.id.clone(), desc.endpoint.clone()),
        ),
    }
}

// ============================================================================
// MOCK ADAPTER (tests only)
// ============================================================================

#[cfg(test)]
#[derive(Debug, Clone)]
pub struct MockLocalAdapter {
    pub runtime_id: String,
    pub endpoint: String,
    pub models: Vec<DiscoveredModel>,
    pub healthy: bool,
    pub fail_invoke: bool,
    pub response_text: String,
}

#[cfg(test)]
impl MockLocalAdapter {
    pub fn healthy(runtime_id: &str) -> Self {
        Self {
            runtime_id: runtime_id.to_string(),
            endpoint: "http://127.0.0.1:11434".to_string(),
            models: vec![DiscoveredModel::basic("mock-local-model")],
            healthy: true,
            fail_invoke: false,
            response_text: "mock local response".to_string(),
        }
    }
}

#[cfg(test)]
#[async_trait]
impl LocalRuntimeAdapter for MockLocalAdapter {
    fn runtime_type(&self) -> LocalRuntimeType {
        LocalRuntimeType::Ollama
    }

    fn endpoint(&self) -> &str {
        &self.endpoint
    }

    async fn probe_health(&self) -> LocalRuntimeHealthReport {
        LocalRuntimeHealthReport {
            runtime_id: self.runtime_id.clone(),
            health: if self.healthy {
                LocalRuntimeHealth::Healthy
            } else {
                LocalRuntimeHealth::Unavailable
            },
            reachable: self.healthy,
            models_available: self.models.len(),
            models_loaded: 0,
            latency_ms: Some(1),
            last_success_timestamp: Some(now_timestamp()),
            last_error: None,
            checked_at_timestamp: now_timestamp(),
        }
    }

    async fn list_models(&self) -> Result<Vec<DiscoveredModel>, LocalRuntimeError> {
        if self.healthy {
            Ok(self.models.clone())
        } else {
            Err(LocalRuntimeError::RuntimeUnavailable {
                reason: "Mock runtime down".to_string(),
            })
        }
    }

    async fn invoke(
        &self,
        _model: &str,
        request: &ModelRequest,
        _timeout: Duration,
    ) -> Result<crate::models::ModelResponse, LocalRuntimeError> {
        if self.fail_invoke {
            return Err(LocalRuntimeError::RuntimeUnavailable {
                reason: "Mock inference failure".to_string(),
            });
        }
        Ok(crate::models::ModelResponse {
            model_id: request.model_id.clone(),
            content: self.response_text.clone(),
            tool_calls: vec![],
            usage: crate::models::Usage::default(),
            finish_reason: crate::models::FinishReason::Stop,
            metadata: HashMap::new(),
        })
    }

    async fn stream(
        &self,
        _model: &str,
        request: &ModelRequest,
        _timeout: Duration,
    ) -> Result<mpsc::Receiver<crate::models::ModelStreamEvent>, LocalRuntimeError> {
        if self.fail_invoke {
            return Err(LocalRuntimeError::RuntimeUnavailable {
                reason: "Mock stream failure".to_string(),
            });
        }
        let (tx, rx) = mpsc::channel(8);
        let resp = self.invoke(_model, request, Duration::from_secs(5)).await?;
        tokio::spawn(async move {
            let _ = tx
                .send(crate::models::ModelStreamEvent::TextDelta(
                    resp.content.clone(),
                ))
                .await;
            let _ = tx
                .send(crate::models::ModelStreamEvent::Completed(resp))
                .await;
        });
        Ok(rx)
    }
}

// ============================================================================
// PROVIDER BRIDGE — exposes a managed local runtime through ModelProvider
// ============================================================================
//
// The bridge is registered in the existing `ProviderRegistry` under
// `local-runtime-<id>` so `ModelRuntime` and `ModelRouter` keep working
// unchanged: local models appear as ordinary `ExecutionMode::Local`
// descriptors. Failures surface as `LocalRuntimeUnavailable` and never
// trigger cloud fallback (enforced by `FallbackGuard` + router policy).
// ============================================================================

pub struct LocalRuntimeProviderBridge {
    provider_id: String,
    display_name: String,
    adapter: Arc<dyn LocalRuntimeAdapter>,
    descriptor_cache: std::sync::RwLock<Vec<ModelDescriptor>>,
}

impl LocalRuntimeProviderBridge {
    pub fn new(runtime: &LocalRuntimeDescriptor, adapter: Arc<dyn LocalRuntimeAdapter>) -> Self {
        Self {
            provider_id: format!("local-runtime-{}", runtime.id),
            display_name: format!("Local {}", runtime.name),
            adapter,
            descriptor_cache: std::sync::RwLock::new(Vec::new()),
        }
    }

    pub fn provider_id(&self) -> &str {
        &self.provider_id
    }

    /// Refresh the cached descriptors from adapter discovery.
    pub async fn refresh_cache(&self) {
        let discovered = self.adapter.list_models().await.unwrap_or_default();
        let mut out = Vec::new();
        for d in discovered {
            out.push(descriptor_from_discovered(
                &self.provider_id,
                self.adapter.endpoint(),
                &d,
            ));
        }
        if let Ok(mut guard) = self.descriptor_cache.write() {
            *guard = out;
        }
    }

    fn model_identifier<'a>(&self, request_id: &'a str) -> &'a str {
        request_id
            .split_once(':')
            .map(|(_, m)| m)
            .unwrap_or(request_id)
    }
}

pub fn descriptor_from_discovered(
    provider_id: &str,
    endpoint: &str,
    d: &DiscoveredModel,
) -> ModelDescriptor {
    let mut metadata = HashMap::new();
    metadata.insert("local_endpoint".to_string(), endpoint.to_string());
    if let Some(q) = &d.quantization {
        metadata.insert("quantization".to_string(), q.clone());
    }
    if let Some(a) = &d.architecture {
        metadata.insert("architecture".to_string(), a.clone());
    }
    if let Some(f) = &d.model_format {
        metadata.insert("model_format".to_string(), f.clone());
    }
    if let Some(p) = d.parameter_count_billions {
        metadata.insert("parameter_count_b".to_string(), format!("{}", p));
    }
    ModelDescriptor {
        id: format!("{}:{}", provider_id, d.model_identifier),
        provider_id: provider_id.to_string(),
        model_identifier: d.model_identifier.clone(),
        display_name: d.display_name.clone(),
        execution_mode: crate::providers::ExecutionMode::Local,
        capabilities: d.capabilities.clone(),
        context_window: d.context_window,
        max_output_tokens: d.max_output_tokens,
        tokenizer: crate::models::TokenizerInfo::Estimated { factor: 1.0 },
        hardware_requirements: match (d.required_ram_mb, d.required_vram_mb) {
            (None, None) => None,
            (ram, vram) => Some(crate::models::HardwareRequirement {
                minimum_ram: ram,
                recommended_ram: None,
                minimum_vram: vram,
                recommended_vram: None,
            }),
        },
        availability: ModelAvailability::Available,
        metadata,
    }
}

#[async_trait]
impl crate::providers::ModelProvider for LocalRuntimeProviderBridge {
    fn provider_info(&self) -> crate::providers::ProviderDescriptor {
        crate::providers::ProviderDescriptor {
            id: self.provider_id.clone(),
            name: self.display_name.clone(),
            provider_type: ProviderType::Ollama,
            execution_mode: crate::providers::ExecutionMode::Local,
            enabled: true,
            status: crate::providers::ProviderStatus::Available,
            base_url: Some(self.adapter.endpoint().to_string()),
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
        self.descriptor_cache
            .read()
            .map(|g| g.clone())
            .unwrap_or_default()
    }

    async fn health_check(&self) -> crate::providers::ProviderHealth {
        let report = self.adapter.probe_health().await;
        let status = match report.health {
            LocalRuntimeHealth::Healthy => crate::providers::ProviderStatus::Available,
            LocalRuntimeHealth::Degraded => crate::providers::ProviderStatus::Degraded,
            LocalRuntimeHealth::Unavailable | LocalRuntimeHealth::Unknown => {
                crate::providers::ProviderStatus::Unavailable
            }
        };
        crate::providers::ProviderHealth {
            provider_id: self.provider_id.clone(),
            status,
            latency_ms: report.latency_ms.unwrap_or(0),
            checked_at_timestamp: report.checked_at_timestamp,
            error_message: report.last_error,
        }
    }

    async fn invoke(
        &self,
        request: &ModelRequest,
    ) -> Result<crate::models::ModelResponse, ModelError> {
        let identifier = self.model_identifier(&request.model_id).to_string();
        self.adapter
            .invoke(&identifier, request, Duration::from_secs(120))
            .await
            .map_err(|e| e.to_model_error(&self.provider_id, &request.model_id))
    }

    async fn stream(
        &self,
        request: &ModelRequest,
    ) -> Result<mpsc::Receiver<crate::models::ModelStreamEvent>, ModelError> {
        let identifier = self.model_identifier(&request.model_id).to_string();
        self.adapter
            .stream(&identifier, request, Duration::from_secs(300))
            .await
            .map_err(|e| e.to_model_error(&self.provider_id, &request.model_id))
    }
}

/// Capability set advertised by a runtime descriptor.
pub fn capabilities_for_type(
    runtime_type: LocalRuntimeType,
    execution_mode: LocalExecutionMode,
) -> Vec<ModelCapability> {
    let _ = execution_mode;
    match runtime_type {
        LocalRuntimeType::Ollama => vec![
            ModelCapability::TextGeneration,
            ModelCapability::CodeGeneration,
            ModelCapability::Streaming,
        ],
        _ => vec![ModelCapability::TextGeneration, ModelCapability::Streaming],
    }
}

/// Default lifecycle for a freshly discovered model.
pub fn initial_state_for_discovery() -> LocalModelState {
    LocalModelState::Discovered
}
