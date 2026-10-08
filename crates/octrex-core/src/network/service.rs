use super::audit::NetworkAuditLogger;
use super::decision::NetworkDecision;
use super::endpoint::NetworkEndpoint;
use super::errors::NetworkError;
use super::evaluator::NetworkPolicyEvaluator;
use super::policy::NetworkRule;
use super::request::NetworkRequest;
use super::types::NetworkMode;
use crate::db::DatabaseManager;
use crate::events::EventBus;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkSecurityStatus {
    pub enabled: bool,
    pub mode: NetworkMode,
    pub policy_version: String,
    pub active_rules_count: usize,
    pub allowlist_count: usize,
    pub total_decisions_count: usize,
    pub blocked_count: usize,
    pub allowed_count: usize,
    pub consent_required_count: usize,
    pub last_decision: Option<NetworkDecision>,
    pub service_health: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConsentRecord {
    pub id: String,
    pub request_id: String,
    pub destination: String,
    pub granted: bool,
    pub timestamp: u64,
}

pub struct NetworkSecurityService {
    evaluator: NetworkPolicyEvaluator,
    audit_logger: Option<NetworkAuditLogger>,
    activity_history: Arc<RwLock<Vec<NetworkDecision>>>,
    consents: Arc<RwLock<Vec<NetworkConsentRecord>>>,
}

impl Default for NetworkSecurityService {
    fn default() -> Self {
        Self::new()
    }
}

impl NetworkSecurityService {
    pub fn new() -> Self {
        Self {
            evaluator: NetworkPolicyEvaluator::new(),
            audit_logger: None,
            activity_history: Arc::new(RwLock::new(Vec::new())),
            consents: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn with_db_and_events(db: Arc<DatabaseManager>, event_bus: Arc<EventBus>) -> Self {
        let audit_logger = NetworkAuditLogger::new(db, event_bus);
        Self {
            evaluator: NetworkPolicyEvaluator::new(),
            audit_logger: Some(audit_logger),
            activity_history: Arc::new(RwLock::new(Vec::new())),
            consents: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn set_mode(&self, mode: NetworkMode) {
        self.evaluator.set_mode(mode);
    }

    pub fn get_mode(&self) -> NetworkMode {
        self.evaluator.get_mode()
    }

    pub fn evaluate_request(&self, request: &NetworkRequest) -> NetworkDecision {
        let decision = self.evaluator.evaluate(request);

        // Audit log
        if let Some(logger) = &self.audit_logger {
            logger.log_decision(request, &decision);
        }

        // Add to in-memory history
        {
            let mut history = self.activity_history.write().unwrap();
            if history.len() >= 1000 {
                history.remove(0);
            }
            history.push(decision.clone());
        }

        decision
    }

    pub fn evaluate_redirect(
        &self,
        current: &NetworkEndpoint,
        next_url: &str,
    ) -> Result<NetworkDecision, NetworkError> {
        self.evaluator.evaluate_redirect(current, next_url)
    }

    pub fn get_status(&self) -> NetworkSecurityStatus {
        let mode = self.evaluator.get_mode();
        let rules = self.evaluator.get_policy_set().rules;
        let allowlist = self.evaluator.get_allowlist().entries;
        let history = self.activity_history.read().unwrap();

        let total_decisions_count = history.len();
        let blocked_count = history
            .iter()
            .filter(|d| d.disposition.is_blocked())
            .count();
        let allowed_count = history
            .iter()
            .filter(|d| d.disposition.is_allowed())
            .count();
        let consent_required_count = history.iter().filter(|d| d.requires_consent).count();
        let last_decision = history.last().cloned();

        NetworkSecurityStatus {
            enabled: mode != NetworkMode::Disabled,
            mode,
            policy_version: "1.0.0".to_string(),
            active_rules_count: rules.iter().filter(|r| r.enabled).count(),
            allowlist_count: allowlist.iter().filter(|a| a.enabled).count(),
            total_decisions_count,
            blocked_count,
            allowed_count,
            consent_required_count,
            last_decision,
            service_health: "HEALTHY".to_string(),
        }
    }

    pub fn get_rules(&self) -> Vec<NetworkRule> {
        self.evaluator.get_policy_set().rules
    }

    pub fn get_allowlist(&self) -> super::allowlist::NetworkAllowlist {
        self.evaluator.get_allowlist()
    }

    pub fn add_allowlist_entry(&self, domain_pattern: &str, description: &str) {
        self.evaluator
            .add_allowlist_entry(domain_pattern, description);
    }

    pub fn remove_allowlist_entry(&self, id: &str) -> bool {
        self.evaluator.remove_allowlist_entry(id)
    }

    pub fn get_decisions(&self) -> Vec<NetworkDecision> {
        let history = self.activity_history.read().unwrap();
        history.clone()
    }

    pub fn get_decision_by_id(&self, id: &str) -> Option<NetworkDecision> {
        let history = self.activity_history.read().unwrap();
        history.iter().find(|d| d.decision_id == id).cloned()
    }

    pub fn record_consent(
        &self,
        request_id: &str,
        destination: &str,
        granted: bool,
    ) -> NetworkConsentRecord {
        let record = NetworkConsentRecord {
            id: format!("consent-{}", uuid::Uuid::new_v4()),
            request_id: request_id.to_string(),
            destination: destination.to_string(),
            granted,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        };

        let mut list = self.consents.write().unwrap();
        list.push(record.clone());
        record
    }
}
