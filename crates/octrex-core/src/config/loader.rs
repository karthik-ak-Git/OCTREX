use crate::config::types::{AppConfig, ProviderConfig, ProviderType};
use std::fs;
use std::path::PathBuf;

pub struct ConfigLoader;

impl ConfigLoader {
    pub fn config_path() -> PathBuf {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        home.join(".octrex").join("config.json")
    }

    pub fn load() -> AppConfig {
        let path = Self::config_path();
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(config) = serde_json::from_str::<AppConfig>(&content) {
                    return config;
                }
            }
        }
        AppConfig::default()
    }

    pub fn save(config: &AppConfig) -> anyhow::Result<()> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(config)?;
        fs::write(path, json)?;
        Ok(())
    }
}

impl AppConfig {
    pub fn config_path() -> PathBuf {
        ConfigLoader::config_path()
    }

    pub fn load() -> Self {
        ConfigLoader::load()
    }

    pub fn save(&self) -> anyhow::Result<()> {
        ConfigLoader::save(self)
    }

    pub fn set_api_key(&mut self, provider: &str, key: String) -> anyhow::Result<()> {
        if let Some(cfg) = self.providers.get_mut(provider) {
            cfg.api_key = Some(key);
            self.save()?;
            Ok(())
        } else {
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
