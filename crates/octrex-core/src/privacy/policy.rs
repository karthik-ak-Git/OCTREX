use super::rules::get_default_policy_rules;
use super::types::{PolicyAction, PolicyRule, PolicySource, PrivacyClassification, PrivacyContext};
use crate::providers::ExecutionMode;
use std::sync::RwLock;

pub struct PolicyEngine {
    rules: RwLock<Vec<PolicyRule>>,
    version: RwLock<u32>,
}

impl PolicyEngine {
    pub fn new() -> Self {
        Self {
            rules: RwLock::new(get_default_policy_rules()),
            version: RwLock::new(1),
        }
    }

    pub fn with_rules(rules: Vec<PolicyRule>) -> Self {
        Self {
            rules: RwLock::new(rules),
            version: RwLock::new(1),
        }
    }

    pub fn current_version(&self) -> u32 {
        *self.version.read().unwrap()
    }

    pub fn list_rules(&self) -> Vec<PolicyRule> {
        let guard = self.rules.read().unwrap();
        guard.clone()
    }

    pub fn add_rule(&self, rule: PolicyRule) {
        let mut guard = self.rules.write().unwrap();
        guard.push(rule);
        let mut ver = self.version.write().unwrap();
        *ver += 1;
    }

    pub fn update_rules(&self, rules: Vec<PolicyRule>) {
        let mut guard = self.rules.write().unwrap();
        *guard = rules;
        let mut ver = self.version.write().unwrap();
        *ver += 1;
    }

    /// Evaluates effective policy for a given PrivacyContext and Data Classification.
    /// Orders rules by PolicySource priority (System > Company > Security > Privacy > User).
    /// Returns winning PolicyAction, matching PolicyRule, and effective PolicySource.
    pub fn evaluate(
        &self,
        context: &PrivacyContext,
        effective_classification: PrivacyClassification,
    ) -> (PolicyAction, PolicyRule, PolicySource) {
        let mut active_rules = {
            let guard = self.rules.read().unwrap();
            guard.clone()
        };

        // Deterministic sort: PolicySource priority DESC, Rule priority DESC
        active_rules.sort_by(|a, b| {
            let source_cmp = (b.source as u8).cmp(&(a.source as u8));
            if source_cmp != std::cmp::Ordering::Equal {
                source_cmp
            } else {
                b.priority.cmp(&a.priority)
            }
        });

        // 1. Mandatory Hard Invariants check before rule scan
        // If confidential mode or strict local mode is requested, block cloud
        if (context.confidential_mode_enabled
            || context.user_privacy_mode == crate::privacy::types::PrivacyMode::Confidential
            || context.user_privacy_mode == crate::privacy::types::PrivacyMode::LocalOnly)
            && context.requested_mode == ExecutionMode::Cloud
        {
            let fallback_rule = PolicyRule {
                id: "SYS-HARD-01".to_string(),
                name: "System Local/Confidential Hard Boundary".to_string(),
                source: PolicySource::System,
                priority: 1000,
                enabled: true,
                scope: crate::privacy::types::PolicyScope::Global,
                target_classification: None,
                target_mode: Some(ExecutionMode::Cloud),
                forbidden_providers: vec![],
                forbidden_models: vec![],
                action: PolicyAction::Deny,
                reason: "System hard boundary: Online processing is strictly forbidden when Confidential or Local-Only privacy mode is active.".to_string(),
                version: self.current_version(),
            };
            return (PolicyAction::Deny, fallback_rule, PolicySource::System);
        }

        // 2. Company / System Policy: Confidential / Restricted / Secret classification prohibits Cloud
        if (effective_classification >= PrivacyClassification::Confidential
            || context.workspace_classification >= PrivacyClassification::Confidential)
            && context.requested_mode == ExecutionMode::Cloud
        {
            let fallback_rule = PolicyRule {
                id: "CMP-HARD-01".to_string(),
                name: "Company Confidentiality Protection Rule".to_string(),
                source: PolicySource::Company,
                priority: 900,
                enabled: true,
                scope: crate::privacy::types::PolicyScope::Global,
                target_classification: Some(effective_classification),
                target_mode: Some(ExecutionMode::Cloud),
                forbidden_providers: vec![],
                forbidden_models: vec![],
                action: PolicyAction::Deny,
                reason: format!("Company policy prohibits online execution for workspace/payload classified as {}.", effective_classification),
                version: self.current_version(),
            };
            return (PolicyAction::Deny, fallback_rule, PolicySource::Company);
        }

        // 3. Scan active registered rules
        for rule in &active_rules {
            if !rule.enabled {
                continue;
            }

            // Check mode match
            if let Some(target_mode) = rule.target_mode {
                if target_mode != context.requested_mode {
                    continue;
                }
            }

            // Check classification match
            if let Some(target_class) = rule.target_classification {
                if effective_classification < target_class {
                    continue;
                }
            }

            // Security credential rule check
            if rule.id == "SEC-001" {
                let has_cred = context.inputs.iter().any(|inp| {
                    inp.content.contains("api_key")
                        || inp.content.contains("sk-")
                        || inp.content.contains("PRIVATE KEY")
                        || inp.content.contains("aws_secret_access_key")
                });
                if !has_cred {
                    continue;
                }
            }

            // Check provider / model forbidden lists
            if let Some(provider) = &context.candidate_provider {
                if rule.forbidden_providers.contains(provider) {
                    return (PolicyAction::Deny, rule.clone(), rule.source);
                }
            }
            if let Some(model) = &context.candidate_model {
                if rule.forbidden_models.contains(model) {
                    return (PolicyAction::Deny, rule.clone(), rule.source);
                }
            }

            // Rule matches!
            return (rule.action, rule.clone(), rule.source);
        }

        // 4. Fail-closed default fallback if no explicit rule matches
        let default_rule = PolicyRule {
            id: "SYS-DEFAULT".to_string(),
            name: "Fail-Closed System Default".to_string(),
            source: PolicySource::System,
            priority: 0,
            enabled: true,
            scope: crate::privacy::types::PolicyScope::Global,
            target_classification: None,
            target_mode: None,
            forbidden_providers: vec![],
            forbidden_models: vec![],
            action: if context.requested_mode == ExecutionMode::Cloud {
                PolicyAction::Deny
            } else {
                PolicyAction::Allow
            },
            reason: "Default fail-closed policy evaluation.".to_string(),
            version: self.current_version(),
        };

        (default_rule.action, default_rule, PolicySource::System)
    }
}

impl Default for PolicyEngine {
    fn default() -> Self {
        Self::new()
    }
}
