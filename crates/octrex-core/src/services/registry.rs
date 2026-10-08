use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ServiceStatus {
    Starting,
    Ready,
    Degraded,
    Stopped,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceDescriptor {
    pub name: String,
    pub description: String,
    pub status: ServiceStatus,
}

#[derive(Clone, Default)]
pub struct ServiceRegistry {
    services: Arc<RwLock<HashMap<String, ServiceDescriptor>>>,
}

impl ServiceRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_service(&self, name: impl Into<String>, description: impl Into<String>) {
        let name_str = name.into();
        let service = ServiceDescriptor {
            name: name_str.clone(),
            description: description.into(),
            status: ServiceStatus::Starting,
        };
        let mut guard = self.services.write().unwrap();
        guard.insert(name_str, service);
    }

    pub fn update_status(&self, name: &str, status: ServiceStatus) -> bool {
        let mut guard = self.services.write().unwrap();
        if let Some(service) = guard.get_mut(name) {
            service.status = status;
            true
        } else {
            false
        }
    }

    pub fn list_services(&self) -> Vec<ServiceDescriptor> {
        let guard = self.services.read().unwrap();
        guard.values().cloned().collect()
    }

    pub fn is_healthy(&self) -> bool {
        let guard = self.services.read().unwrap();
        guard
            .values()
            .all(|s| s.status == ServiceStatus::Ready || s.status == ServiceStatus::Starting)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_registry() {
        let registry = ServiceRegistry::new();
        registry.register_service("workspace_service", "Filesystem and workspace manager");
        assert!(registry.is_healthy());

        registry.update_status("workspace_service", ServiceStatus::Ready);
        assert_eq!(registry.list_services()[0].status, ServiceStatus::Ready);
    }
}
