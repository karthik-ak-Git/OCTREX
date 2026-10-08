use crate::app::ApplicationState;
use crate::ipc::responses::{ApplicationInfo, BackendHealth, ConfigurationSummary, RuntimeStatus};
use std::collections::HashMap;
use std::time::Instant;

pub struct IpcCommandHandler;

impl IpcCommandHandler {
    pub fn get_application_info() -> ApplicationInfo {
        ApplicationInfo {
            name: "OCTREX CODE V4".to_string(),
            version: "4.0.0".to_string(),
            backend_version: "4.0.0".to_string(),
            protocol_version: "1.0.0".to_string(),
            environment: if cfg!(debug_assertions) {
                "development".to_string()
            } else {
                "production".to_string()
            },
        }
    }

    pub fn get_runtime_status(state: &ApplicationState, start_time: Instant) -> RuntimeStatus {
        RuntimeStatus {
            state: format!("{:?}", state.lifecycle.current_state()),
            uptime_ms: start_time.elapsed().as_millis(),
            active_tasks: state.task_registry.list_tasks().len(),
            active_sessions: state.session_registry.list_sessions().len(),
            active_workspaces: state.workspace_registry.list_workspaces().len(),
        }
    }

    pub fn get_backend_health(state: &ApplicationState) -> BackendHealth {
        let is_healthy = state.service_registry.is_healthy() && state.lifecycle.is_ready();
        let mut subsystems = HashMap::new();

        for service in state.service_registry.list_services() {
            subsystems.insert(service.name, format!("{:?}", service.status));
        }

        BackendHealth {
            status: if is_healthy {
                "HEALTHY".to_string()
            } else {
                "DEGRADED".to_string()
            },
            version: "4.0.0".to_string(),
            runtime: "octrex-core-rust".to_string(),
            initialized: state.lifecycle.is_ready(),
            subsystems,
        }
    }

    pub fn get_configuration_summary(state: &ApplicationState) -> ConfigurationSummary {
        let guard = state.config.read().unwrap();
        ConfigurationSummary {
            active_provider: guard.active_provider.clone(),
            active_workspace: guard
                .active_workspace
                .as_ref()
                .map(|p| p.to_string_lossy().to_string()),
            provider_count: guard.providers.len(),
            privacy_mode: guard.security.privacy_mode.clone(),
            enforcement_level: guard.security.enforcement_level.clone(),
        }
    }
}
