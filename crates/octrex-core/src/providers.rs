use crate::config::{AppConfig, ProviderConfig, ProviderType};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

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

pub struct ProviderGateway {
    http_client: Client,
}

impl Default for ProviderGateway {
    fn default() -> Self {
        let http_client = Client::builder()
            .timeout(Duration::from_secs(8))
            .build()
            .unwrap_or_else(|_| Client::new());
        Self { http_client }
    }
}

impl ProviderGateway {
    pub fn new() -> Self {
        Self::default()
    }

    /// Check live health status against actual remote / local provider endpoints
    pub async fn check_health(
        &self,
        provider_id: &str,
        config: &ProviderConfig,
    ) -> ProviderHealthCheck {
        let start = std::time::Instant::now();
        let status = match config.provider_type {
            ProviderType::Ollama => self.check_ollama(config).await,
            ProviderType::OpenAI | ProviderType::CraxGpt | ProviderType::OpenRouter | ProviderType::Groq => {
                self.check_openai_compatible(config).await
            }
            ProviderType::Anthropic => self.check_anthropic(config).await,
            ProviderType::Google => self.check_google(config).await,
            ProviderType::Custom => self.check_openai_compatible(config).await,
        };

        let latency_ms = start.elapsed().as_millis();
        ProviderHealthCheck {
            provider_id: provider_id.to_string(),
            status,
            latency_ms,
        }
    }

    async fn check_ollama(&self, config: &ProviderConfig) -> ProviderHealthStatus {
        let base_url = config
            .base_url
            .as_deref()
            .unwrap_or("http://127.0.0.1:11434");
        let endpoint = format!("{}/api/tags", base_url.trim_end_matches('/'));

        match self.http_client.get(&endpoint).send().await {
            Ok(resp) => {
                if resp.status().is_success() {
                    #[derive(Deserialize)]
                    struct OllamaModel {
                        name: String,
                    }
                    #[derive(Deserialize)]
                    struct OllamaResponse {
                        models: Vec<OllamaModel>,
                    }

                    if let Ok(data) = resp.json::<OllamaResponse>().await {
                        let model_names = data.models.into_iter().map(|m| m.name).collect();
                        ProviderHealthStatus::Connected { models: model_names }
                    } else {
                        ProviderHealthStatus::Connected {
                            models: vec![config.default_model.clone()],
                        }
                    }
                } else {
                    ProviderHealthStatus::Offline {
                        reason: format!("HTTP {}", resp.status()),
                    }
                }
            }
            Err(e) => ProviderHealthStatus::Offline {
                reason: e.to_string(),
            },
        }
    }

    async fn check_openai_compatible(&self, config: &ProviderConfig) -> ProviderHealthStatus {
        let api_key = match &config.api_key {
            Some(key) if !key.trim().is_empty() => key.trim(),
            _ => return ProviderHealthStatus::MissingApiKey,
        };

        let base_url = config
            .base_url
            .as_deref()
            .unwrap_or("https://api.openai.com/v1");
        let endpoint = format!("{}/models", base_url.trim_end_matches('/'));

        let request = self
            .http_client
            .get(&endpoint)
            .bearer_auth(api_key);

        match request.send().await {
            Ok(resp) => {
                let status_code = resp.status();
                if status_code.is_success() {
                    #[derive(Deserialize)]
                    struct ModelItem {
                        id: String,
                    }
                    #[derive(Deserialize)]
                    struct ModelsResponse {
                        data: Vec<ModelItem>,
                    }

                    if let Ok(parsed) = resp.json::<ModelsResponse>().await {
                        let models = parsed.data.into_iter().map(|m| m.id).take(20).collect();
                        ProviderHealthStatus::Connected { models }
                    } else {
                        ProviderHealthStatus::Connected {
                            models: vec![config.default_model.clone()],
                        }
                    }
                } else if status_code.as_u16() == 401 || status_code.as_u16() == 403 {
                    ProviderHealthStatus::AuthError {
                        message: format!("Authentication failed (HTTP {})", status_code),
                    }
                } else {
                    ProviderHealthStatus::Offline {
                        reason: format!("HTTP {}", status_code),
                    }
                }
            }
            Err(e) => ProviderHealthStatus::Offline {
                reason: e.to_string(),
            },
        }
    }

    async fn check_anthropic(&self, config: &ProviderConfig) -> ProviderHealthStatus {
        let api_key = match &config.api_key {
            Some(key) if !key.trim().is_empty() => key.trim(),
            _ => return ProviderHealthStatus::MissingApiKey,
        };

        let endpoint = "https://api.anthropic.com/v1/messages";
        let payload = serde_json::json!({
            "model": "claude-3-5-haiku-20241022",
            "max_tokens": 1,
            "messages": [{"role": "user", "content": "ping"}]
        });

        match self
            .http_client
            .post(endpoint)
            .header("x-api-key", api_key)
            .header("anthropic-version", "2023-06-01")
            .json(&payload)
            .send()
            .await
        {
            Ok(resp) => {
                let status_code = resp.status();
                if status_code.is_success() || status_code.as_u16() == 429 {
                    ProviderHealthStatus::Connected {
                        models: vec![
                            "claude-3-5-sonnet-20241022".to_string(),
                            "claude-3-5-haiku-20241022".to_string(),
                            "claude-3-opus-20240229".to_string(),
                        ],
                    }
                } else if status_code.as_u16() == 401 || status_code.as_u16() == 403 {
                    ProviderHealthStatus::AuthError {
                        message: format!("Invalid Anthropic API Key (HTTP {})", status_code),
                    }
                } else {
                    ProviderHealthStatus::Offline {
                        reason: format!("HTTP {}", status_code),
                    }
                }
            }
            Err(e) => ProviderHealthStatus::Offline {
                reason: e.to_string(),
            },
        }
    }

    async fn check_google(&self, config: &ProviderConfig) -> ProviderHealthStatus {
        let api_key = match &config.api_key {
            Some(key) if !key.trim().is_empty() => key.trim(),
            _ => return ProviderHealthStatus::MissingApiKey,
        };

        let endpoint = format!(
            "https://generativelanguage.googleapis.com/v1beta/models?key={}",
            api_key
        );

        match self.http_client.get(&endpoint).send().await {
            Ok(resp) => {
                let status_code = resp.status();
                if status_code.is_success() {
                    #[derive(Deserialize)]
                    struct GeminiModel {
                        name: String,
                    }
                    #[derive(Deserialize)]
                    struct GeminiResponse {
                        models: Vec<GeminiModel>,
                    }

                    if let Ok(parsed) = resp.json::<GeminiResponse>().await {
                        let models = parsed
                            .models
                            .into_iter()
                            .map(|m| m.name.trim_start_matches("models/").to_string())
                            .collect();
                        ProviderHealthStatus::Connected { models }
                    } else {
                        ProviderHealthStatus::Connected {
                            models: vec!["gemini-1.5-pro".to_string(), "gemini-1.5-flash".to_string()],
                        }
                    }
                } else if status_code.as_u16() == 400 || status_code.as_u16() == 403 {
                    ProviderHealthStatus::AuthError {
                        message: format!("Invalid Gemini API Key (HTTP {})", status_code),
                    }
                } else {
                    ProviderHealthStatus::Offline {
                        reason: format!("HTTP {}", status_code),
                    }
                }
            }
            Err(e) => ProviderHealthStatus::Offline {
                reason: e.to_string(),
            },
        }
    }

    pub async fn check_all(&self, config: &AppConfig) -> Vec<ProviderHealthCheck> {
        let mut results = Vec::new();
        for (id, pcfg) in &config.providers {
            let res = self.check_health(id, pcfg).await;
            results.push(res);
        }
        results
    }
}
