use super::allowlist::NetworkAllowlist;
use super::decision::{Disposition, NetworkDecision, PolicySource};
use super::endpoint::NetworkEndpoint;
use super::errors::NetworkError;
use super::policy::NetworkPolicySet;
use super::request::NetworkRequest;
use super::resolver::SafeDnsResolver;
use super::types::NetworkMode;
use super::validation::is_cloud_metadata_endpoint;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluatorConfig {
    pub mode: NetworkMode,
    pub allowlist_enabled: bool,
    pub ssrf_protection_enabled: bool,
}

impl Default for EvaluatorConfig {
    fn default() -> Self {
        Self {
            mode: NetworkMode::LocalOnly,
            allowlist_enabled: true,
            ssrf_protection_enabled: true,
        }
    }
}

pub struct NetworkPolicyEvaluator {
    config: Arc<RwLock<EvaluatorConfig>>,
    policies: Arc<RwLock<NetworkPolicySet>>,
    allowlist: Arc<RwLock<NetworkAllowlist>>,
    dns_resolver: SafeDnsResolver,
}

impl Default for NetworkPolicyEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

impl NetworkPolicyEvaluator {
    pub fn new() -> Self {
        Self {
            config: Arc::new(RwLock::new(EvaluatorConfig::default())),
            policies: Arc::new(RwLock::new(NetworkPolicySet::default())),
            allowlist: Arc::new(RwLock::new(NetworkAllowlist::new())),
            dns_resolver: SafeDnsResolver::new(),
        }
    }

    pub fn set_mode(&self, mode: NetworkMode) {
        let mut cfg = self.config.write().unwrap();
        cfg.mode = mode;
    }

    pub fn get_mode(&self) -> NetworkMode {
        let cfg = self.config.read().unwrap();
        cfg.mode
    }

    pub fn get_allowlist(&self) -> NetworkAllowlist {
        self.allowlist.read().unwrap().clone()
    }

    pub fn get_policy_set(&self) -> NetworkPolicySet {
        self.policies.read().unwrap().clone()
    }

    pub fn add_allowlist_entry(&self, domain_pattern: &str, description: &str) {
        let mut al = self.allowlist.write().unwrap();
        al.add(domain_pattern, description);
    }

    pub fn remove_allowlist_entry(&self, id: &str) -> bool {
        let mut al = self.allowlist.write().unwrap();
        al.remove(id)
    }

    /// Evaluates a NetworkRequest against mode, allowlist, SSRF, DNS, and policy rules.
    /// FAILS CLOSED: Any unknown, error, or unpermitted request returns BLOCK.
    pub fn evaluate(&self, request: &NetworkRequest) -> NetworkDecision {
        let mode = self.get_mode();

        // 1. Mode check: Disabled
        if mode == NetworkMode::Disabled {
            return NetworkDecision::block(
                "Network access is Disabled",
                None,
                PolicySource::System,
                request.endpoint.clone(),
            );
        }

        // 2. SSRF Check: Cloud metadata endpoints
        if is_cloud_metadata_endpoint(&request.endpoint.host) {
            return NetworkDecision::block(
                "Cloud metadata endpoint blocked (SSRF Protection)",
                None,
                PolicySource::Security,
                request.endpoint.clone(),
            );
        }

        // 3. Target host IP classification check
        let is_loopback = request.endpoint.host == "localhost"
            || request.endpoint.host == "127.0.0.1"
            || request.endpoint.host == "::1";

        // 4. Local-Only Mode Enforcement
        if mode == NetworkMode::LocalOnly && !is_loopback {
            return NetworkDecision::block(
                "Blocked by Local-Only Policy: External network requests are strictly prohibited",
                None,
                PolicySource::System,
                request.endpoint.clone(),
            );
        }

        // 5. DNS Resolution & SSRF Validation (for non-loopback hosts)
        if !is_loopback {
            match self
                .dns_resolver
                .resolve_and_validate(&request.endpoint.host, request.endpoint.port)
            {
                Ok(ips) => {
                    if let Err(err) = self
                        .dns_resolver
                        .check_external_safety(&request.endpoint.host, &ips)
                    {
                        return NetworkDecision::block(
                            format!("Blocked by DNS SSRF Protection: {}", err),
                            None,
                            PolicySource::Security,
                            request.endpoint.clone(),
                        );
                    }
                }
                Err(err) => {
                    return NetworkDecision::block(
                        format!("Fail-Closed: DNS resolution failed ({})", err),
                        None,
                        PolicySource::Security,
                        request.endpoint.clone(),
                    );
                }
            }
        }

        // 6. Restricted Mode Allowlist Check
        if mode == NetworkMode::Restricted && !is_loopback {
            let al = self.allowlist.read().unwrap();
            if !al.is_allowed(&request.endpoint.host) {
                return NetworkDecision::block(
                    format!(
                        "Blocked by Restricted Policy: Host '{}' is not in approved allowlist",
                        request.endpoint.host
                    ),
                    None,
                    PolicySource::Company,
                    request.endpoint.clone(),
                );
            }
        }

        // 7. Policy Rules Evaluation (System > Company > Security > Privacy > Permission > User)
        let policy_set = self.policies.read().unwrap();
        if let Some(rule) = policy_set.evaluate(&request.endpoint, request.capability) {
            match rule.action {
                Disposition::Allow => {
                    return NetworkDecision::allow(
                        format!("Allowed by rule: {}", rule.name),
                        Some(rule.id),
                        rule.source,
                        request.endpoint.clone(),
                    );
                }
                Disposition::Block => {
                    return NetworkDecision::block(
                        format!("Blocked by rule: {}", rule.name),
                        Some(rule.id),
                        rule.source,
                        request.endpoint.clone(),
                    );
                }
                Disposition::RequireConsent => {
                    return NetworkDecision::require_consent(
                        format!("User consent required by rule: {}", rule.name),
                        Some(rule.id),
                        rule.source,
                        request.endpoint.clone(),
                    );
                }
                Disposition::Unknown => {
                    return NetworkDecision::block(
                        "Fail-Closed: Policy rule returned Unknown disposition",
                        Some(rule.id),
                        rule.source,
                        request.endpoint.clone(),
                    );
                }
            }
        }

        // 8. If OnlineAllowed mode and host is in allowlist, allow by default
        if (mode == NetworkMode::OnlineAllowed || mode == NetworkMode::Restricted) && !is_loopback {
            let al = self.allowlist.read().unwrap();
            if al.is_allowed(&request.endpoint.host) {
                return NetworkDecision::allow(
                    format!(
                        "Allowed: Host '{}' is in approved allowlist",
                        request.endpoint.host
                    ),
                    None,
                    PolicySource::Security,
                    request.endpoint.clone(),
                );
            }
        }

        // 9. Loopback allowed for local provider
        if is_loopback {
            return NetworkDecision::allow(
                "Allowed: Local loopback request permitted by security policy",
                None,
                PolicySource::System,
                request.endpoint.clone(),
            );
        }

        // 10. Default FAIL-CLOSED Rule: No matching allow rule => BLOCK
        NetworkDecision::block(
            "Fail-Closed: No policy rule allowed destination (Private by default)",
            None,
            PolicySource::Security,
            request.endpoint.clone(),
        )
    }

    /// Evaluates redirect hops independently. If any hop is blocked, returns BLOCK.
    pub fn evaluate_redirect(
        &self,
        current: &NetworkEndpoint,
        next_url: &str,
    ) -> Result<NetworkDecision, NetworkError> {
        let next_ep = NetworkEndpoint::parse(next_url)?;
        let request = NetworkRequest::new(
            "redirect_hop",
            super::types::NetworkCapability::ExternalHttps,
            next_ep,
        );
        let decision = self.evaluate(&request);
        if !decision.is_allowed() {
            return Err(NetworkError::RedirectBlocked {
                from: current.to_string(),
                to: next_url.to_string(),
                reason: decision.reason,
            });
        }
        Ok(decision)
    }
}
