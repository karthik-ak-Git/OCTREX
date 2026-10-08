use serde::{Deserialize, Serialize};
use std::collections::HashMap;
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
pub struct RuntimeConfig {
    pub host: String,
    pub port: u16,
    pub protocol_version: String,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 3000,
            protocol_version: "1.0.0".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub base_dir: PathBuf,
    pub db_path: PathBuf,
}

impl Default for StorageConfig {
    fn default() -> Self {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        let base = home.join(".octrex");
        Self {
            db_path: base.join("octrex.db"),
            base_dir: base,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub allow_offline: bool,
    pub proxy_url: Option<String>,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            allow_offline: true,
            proxy_url: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub enforcement_level: String,
    pub privacy_mode: String,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            enforcement_level: "standard".to_string(),
            privacy_mode: "strict_local".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacyConfig {
    pub mode: String,
    pub confidential_mode: bool,
    pub consent_behavior: String,
    pub enforcement_level: String,
}

impl Default for PrivacyConfig {
    fn default() -> Self {
        Self {
            mode: "local_only".to_string(),
            confidential_mode: false,
            consent_behavior: "require_consent".to_string(),
            enforcement_level: "strict".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub runtime: RuntimeConfig,
    pub storage: StorageConfig,
    pub network: NetworkConfig,
    pub security: SecurityConfig,
    pub privacy: PrivacyConfig,
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
                api_key: std::env::var("OPENCODE_API_KEY")
                    .ok()
                    .or(Some("free-opencode-router".to_string())),
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
            runtime: RuntimeConfig::default(),
            storage: StorageConfig::default(),
            network: NetworkConfig::default(),
            security: SecurityConfig::default(),
            privacy: PrivacyConfig::default(),
            providers,
            active_provider: "opencode".to_string(),
            active_workspace: None,
        }
    }
}
