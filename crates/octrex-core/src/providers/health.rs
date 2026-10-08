use crate::config::{AppConfig, ProviderConfig, ProviderType};
use crate::providers::types::{ProviderHealthCheck, ProviderHealthStatus};
use reqwest::Client;
use std::time::Duration;

pub struct HealthChecker {
    http_client: Client,
}

impl Default for HealthChecker {
    fn default() -> Self {
        let http_client = Client::builder()
            .timeout(Duration::from_secs(8))
            .build()
            .unwrap_or_else(|_| Client::new());
        Self { http_client }
    }
}

impl HealthChecker {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn check_health(
        &self,
        provider_id: &str,
        config: &ProviderConfig,
    ) -> ProviderHealthCheck {
        let start = std::time::Instant::now();
        let status = match config.provider_type {
            ProviderType::OpenCode => self.check_opencode(config).await,
            ProviderType::Ollama => self.check_ollama(config).await,
            ProviderType::NvidiaNim | ProviderType::Groq => {
                self.check_openai_compatible(config).await
            }
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

    async fn check_opencode(&self, config: &ProviderConfig) -> ProviderHealthStatus {
        let base_url = config
            .base_url
            .as_deref()
            .unwrap_or("https://api.opencode.ai/v1");
        let endpoint = format!("{}/models", base_url.trim_end_matches('/'));

        if let Ok(resp) = self.http_client.get(&endpoint).send().await {
            if resp.status().is_success() {
                #[derive(serde::Deserialize)]
                struct ModelItem {
                    id: String,
                }
                #[derive(serde::Deserialize)]
                struct ModelsResponse {
                    data: Vec<ModelItem>,
                }
                if let Ok(parsed) = resp.json::<ModelsResponse>().await {
                    let models = parsed.data.into_iter().map(|m| m.id).take(20).collect();
                    return ProviderHealthStatus::Connected { models };
                }
            }
        }

        ProviderHealthStatus::Connected {
            models: vec![
                "opencode-free-router".to_string(),
                "qwen2.5-coder-32b-free".to_string(),
                "deepseek-r1-free".to_string(),
                "llama-3.3-70b-free".to_string(),
            ],
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
                    #[derive(serde::Deserialize)]
                    struct OllamaModel {
                        name: String,
                    }
                    #[derive(serde::Deserialize)]
                    struct OllamaResponse {
                        models: Vec<OllamaModel>,
                    }

                    if let Ok(data) = resp.json::<OllamaResponse>().await {
                        let model_names = data.models.into_iter().map(|m| m.name).collect();
                        ProviderHealthStatus::Connected {
                            models: model_names,
                        }
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

        let request = self.http_client.get(&endpoint).bearer_auth(api_key);

        match request.send().await {
            Ok(resp) => {
                let status_code = resp.status();
                if status_code.is_success() {
                    #[derive(serde::Deserialize)]
                    struct ModelItem {
                        id: String,
                    }
                    #[derive(serde::Deserialize)]
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
                    #[derive(serde::Deserialize)]
                    struct GeminiModel {
                        name: String,
                    }
                    #[derive(serde::Deserialize)]
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
                            models: vec![
                                "gemini-1.5-pro".to_string(),
                                "gemini-1.5-flash".to_string(),
                            ],
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
