use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProviderType {
    OpenCode,
    NvidiaNim,
    Google,
    Groq,
    Ollama,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub provider_type: ProviderType,
    pub api_key: Option<String>,
    pub base_url: Option<String>,
    pub default_model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub providers: HashMap<String, ProviderConfig>,
    pub active_provider: String,
    pub active_workspace: Option<PathBuf>,
}

impl Default for AppConfig {
    fn default() -> Self {
        let mut providers = HashMap::new();
        
        providers.insert(
            "opencode".to_string(),
            ProviderConfig {
                provider_type: ProviderType::OpenCode,
                api_key: std::env::var("OPENCODE_API_KEY").ok().or(Some("free-opencode-router".to_string())),
                base_url: Some("https://api.opencode.ai/v1".to_string()),
                default_model: "opencode-free-router".to_string(),
            },
        );

        providers.insert(
            "nvidia-nim".to_string(),
            ProviderConfig {
                provider_type: ProviderType::NvidiaNim,
                api_key: std::env::var("NVIDIA_API_KEY").ok(),
                base_url: Some("https://integrate.api.nvidia.com/v1".to_string()),
                default_model: "meta/llama-3.3-70b-instruct".to_string(),
            },
        );

        providers.insert(
            "google".to_string(),
            ProviderConfig {
                provider_type: ProviderType::Google,
                api_key: std::env::var("GEMINI_API_KEY").ok(),
                base_url: Some("https://generativelanguage.googleapis.com".to_string()),
                default_model: "gemini-1.5-pro".to_string(),
            },
        );

        providers.insert(
            "groq".to_string(),
            ProviderConfig {
                provider_type: ProviderType::Groq,
                api_key: std::env::var("GROQ_API_KEY").ok(),
                base_url: Some("https://api.groq.com/openai/v1".to_string()),
                default_model: "llama-3.3-70b-versatile".to_string(),
            },
        );

        providers.insert(
            "local".to_string(),
            ProviderConfig {
                provider_type: ProviderType::Ollama,
                api_key: None,
                base_url: Some("http://127.0.0.1:11434".to_string()),
                default_model: "llama3".to_string(),
            },
        );

        Self {
            providers,
            active_provider: "opencode".to_string(),
            active_workspace: None,
        }
    }
}

impl AppConfig {
    pub fn config_path() -> PathBuf {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        home.join(".octrex").join("config.json")
    }

    pub fn load() -> Self {
        let path = Self::config_path();
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(config) = serde_json::from_str::<AppConfig>(&content) {
                    return config;
                }
            }
        }
        Self::default()
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)?;
        fs::write(path, json)?;
        Ok(())
    }

    pub fn set_api_key(&mut self, provider: &str, key: String) -> anyhow::Result<()> {
        if let Some(cfg) = self.providers.get_mut(provider) {
            cfg.api_key = Some(key);
            self.save()?;
            Ok(())
        } else {
            // Allow auto-insert if unknown
            let ptype = match provider {
                "opencode" => ProviderType::OpenCode,
                "nvidia-nim" => ProviderType::NvidiaNim,
                "google" => ProviderType::Google,
                "groq" => ProviderType::Groq,
                "local" => ProviderType::Ollama,
                _ => ProviderType::Custom,
            };
            self.providers.insert(
                provider.to_string(),
                ProviderConfig {
                    provider_type: ptype,
                    api_key: Some(key),
                    base_url: None,
                    default_model: "default".to_string(),
                },
            );
            self.save()?;
            Ok(())
        }
    }
}
