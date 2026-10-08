use super::decision::{Disposition, PolicySource};
use super::endpoint::NetworkEndpoint;
use super::types::{NetworkCapability, NetworkProtocol};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkRule {
    pub id: String,
    pub name: String,
    pub source: PolicySource,
    pub action: Disposition,
    pub domain_pattern: String,
    pub protocol: Option<NetworkProtocol>,
    pub port: Option<u16>,
    pub capability: Option<NetworkCapability>,
    pub provider_id: Option<String>,
    pub execution_mode_restriction: Option<String>,
    pub priority: u32,
    pub enabled: bool,
    pub description: String,
}

impl NetworkRule {
    pub fn matches(&self, endpoint: &NetworkEndpoint, capability: NetworkCapability) -> bool {
        if !self.enabled {
            return false;
        }

        // Domain matching
        if !endpoint.matches_domain_pattern(&self.domain_pattern) {
            return false;
        }

        // Protocol matching
        if let Some(proto) = self.protocol {
            if endpoint.protocol != proto {
                return false;
            }
        }

        // Port matching
        if let Some(port) = self.port {
            if endpoint.port != port {
                return false;
            }
        }

        // Capability matching
        if let Some(cap) = self.capability {
            if capability != cap {
                return false;
            }
        }

        true
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkPolicySet {
    pub rules: Vec<NetworkRule>,
}

impl Default for NetworkPolicySet {
    fn default() -> Self {
        Self {
            rules: vec![
                // Default System Policy Rules (Highest Priority)
                NetworkRule {
                    id: "sys-block-loopback-external".to_string(),
                    name: "Block External Loopback Access".to_string(),
                    source: PolicySource::System,
                    action: Disposition::Block,
                    domain_pattern: "127.0.0.1".to_string(),
                    protocol: None,
                    port: None,
                    capability: Some(NetworkCapability::ExternalHttps),
                    provider_id: None,
                    execution_mode_restriction: None,
                    priority: 100,
                    enabled: true,
                    description: "System rule blocking external capability to loopback".to_string(),
                },
                NetworkRule {
                    id: "sys-allow-local-ollama".to_string(),
                    name: "Allow Local Ollama Endpoint".to_string(),
                    source: PolicySource::System,
                    action: Disposition::Allow,
                    domain_pattern: "localhost".to_string(),
                    protocol: Some(NetworkProtocol::Http),
                    port: Some(11434),
                    capability: Some(NetworkCapability::Loopback),
                    provider_id: Some("local".to_string()),
                    execution_mode_restriction: Some("local".to_string()),
                    priority: 90,
                    enabled: true,
                    description: "System rule allowing local Ollama model runtime on port 11434"
                        .to_string(),
                },
            ],
        }
    }
}

impl NetworkPolicySet {
    /// Evaluates rules against policy hierarchy.
    /// Higher priority sources (System > Company > Security > Privacy > Permission > User)
    /// evaluate first. If ANY higher-priority rule matches BLOCK, it immediately returns BLOCK.
    pub fn evaluate(
        &self,
        endpoint: &NetworkEndpoint,
        capability: NetworkCapability,
    ) -> Option<NetworkRule> {
        let mut matching_rules: Vec<_> = self
            .rules
            .iter()
            .filter(|r| r.matches(endpoint, capability))
            .collect();

        // Sort by policy hierarchy priority (System=1 first) then by rule priority descending
        matching_rules.sort_by(|a, b| {
            a.source
                .cmp(&b.source)
                .then_with(|| b.priority.cmp(&a.priority))
        });

        matching_rules.first().cloned().cloned()
    }
}
