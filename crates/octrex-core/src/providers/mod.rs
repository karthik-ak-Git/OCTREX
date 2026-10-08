pub mod adapters;
pub mod gateway;
pub mod health;
pub mod registry;
pub mod types;

#[cfg(test)]
pub mod tests;

#[cfg(test)]
pub use adapters::MockAdapter;
pub use adapters::{GoogleAdapter, OllamaAdapter, OpenAICompatibleAdapter};

pub use gateway::ProviderGateway;
pub use health::HealthChecker;
pub use registry::ProviderRegistry;
pub use types::{
    ExecutionMode, ModelProvider, ProviderDescriptor, ProviderHealth, ProviderHealthCheck,
    ProviderHealthStatus, ProviderId, ProviderStatus,
};
