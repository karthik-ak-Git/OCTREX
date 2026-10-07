use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProviderType {
    Anthropic,
    OpenAI,
    Google,
    Groq,
    Ollama,
    OpenRouter,
    CraxGpt,
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
            "anthropic".to_string(),
            ProviderConfig {
                provider_type: ProviderType::Anthropic,
                api_key: std::env::var("ANTHROPIC_API_KEY").ok(),
                base_url: Some("https://api.anthropic.com".to_string()),
                default_model: "claude-3-5-sonnet-20241022".to_string(),
            },
        );

        providers.insert(
            "openai".to_string(),
            ProviderConfig {
                provider_type: ProviderType::OpenAI,
                api_key: std::env::var("OPENAI_API_KEY").ok(),
                base_url: Some("https://api.openai.com/v1".to_string()),
                default_model: "gpt-4o".to_string(),
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
            "ollama".to_string(),
            ProviderConfig {
                provider_type: ProviderType::Ollama,
                api_key: None,
                base_url: Some("http://127.0.0.1:11434".to_string()),
                default_model: "llama3".to_string(),
            },
        );

        providers.insert(
            "crax-gpt".to_string(),
            ProviderConfig {
                provider_type: ProviderType::CraxGpt,
                api_key: std::env::var("CRAX_GPT_API_KEY").ok(),
                base_url: Some("https://gpt.crax.lol/v1".to_string()),
                default_model: "glm-5.3".to_string(),
            },
        );

        Self {
            providers,
            active_provider: "anthropic".to_string(),
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
            anyhow::bail!("Provider not found: {}", provider)
        }
    }
}
