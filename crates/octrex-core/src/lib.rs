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
            provider_type: ProviderType::Anthropic,
            api_key: None,
            base_url: Some("https://api.anthropic.com".to_string()),
            default_model: "claude-3-5-sonnet".to_string(),
        };

        let check = gateway.check_health("anthropic", &config).await;
        assert_eq!(check.provider_id, "anthropic");
        assert_eq!(check.status, ProviderHealthStatus::MissingApiKey);
    }

    #[tokio::test]
    async fn test_provider_health_check_returns_real_auth_error_for_invalid_key() {
        let gateway = ProviderGateway::new();
        let config = ProviderConfig {
            provider_type: ProviderType::OpenAI,
            api_key: Some("sk-invalid-fake-key-for-test".to_string()),
            base_url: Some("https://api.openai.com/v1".to_string()),
            default_model: "gpt-4o".to_string(),
        };

        let check = gateway.check_health("openai", &config).await;
        assert_eq!(check.provider_id, "openai");
        match check.status {
            ProviderHealthStatus::AuthError { message } => {
                assert!(message.contains("401") || message.contains("Authentication failed"));
            }
            other => panic!("Expected AuthError for invalid key, got: {:?}", other),
        }
    }
}

