pub mod agent;
pub mod config;
pub mod providers;
pub mod workspace;

pub use agent::{AgentEngine, AgentExecutionRequest, AgentExecutionResponse};
pub use config::{AppConfig, ProviderConfig, ProviderType};
pub use providers::{ProviderGateway, ProviderHealthCheck, ProviderHealthStatus};
pub use workspace::{WorkspaceInfo, WorkspaceManager};

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_workspace_manager_inspects_real_disk_directory() {
        let current_dir = std::env::current_dir().expect("Failed to get current dir");
        let info = WorkspaceManager::inspect(&current_dir);
        assert!(info.exists, "Workspace path should exist");
        assert!(info.is_dir, "Workspace path should be a directory");
        assert!(info.total_files > 0, "Current directory should contain files");
    }

    #[tokio::test]
    async fn test_provider_health_check_returns_real_status_for_missing_key() {
        let gateway = ProviderGateway::new();
        let config = ProviderConfig {
            provider_type: ProviderType::NvidiaNim,
            api_key: None,
            base_url: Some("https://integrate.api.nvidia.com/v1".to_string()),
            default_model: "meta/llama-3.3-70b-instruct".to_string(),
        };

        let check = gateway.check_health("nvidia-nim", &config).await;
        assert_eq!(check.provider_id, "nvidia-nim");
        assert_eq!(check.status, ProviderHealthStatus::MissingApiKey);
    }

    #[tokio::test]
    async fn test_provider_health_check_returns_real_auth_error_for_invalid_key() {
        let gateway = ProviderGateway::new();
        let config = ProviderConfig {
            provider_type: ProviderType::Groq,
            api_key: Some("gsk_invalid_fake_key_for_test".to_string()),
            base_url: Some("https://api.groq.com/openai/v1".to_string()),
            default_model: "llama-3.3-70b-versatile".to_string(),
        };

        let check = gateway.check_health("groq", &config).await;
        assert_eq!(check.provider_id, "groq");
        match check.status {
            ProviderHealthStatus::AuthError { message } => {
                assert!(message.contains("401") || message.contains("Authentication failed"));
            }
            other => panic!("Expected AuthError for invalid key, got: {:?}", other),
        }
    }
}

