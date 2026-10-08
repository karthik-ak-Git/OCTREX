use crate::context::errors::ContextError;
use crate::context::item::ContextItem;
use crate::context::types::ContextRole;
use crate::ids::{SessionId, WorkspaceId};
use crate::privacy::PrivacyClassification;

/// Security boundary enforcer for Context Engine
pub struct ContextSecurityPolicy;

impl ContextSecurityPolicy {
    /// Validates a single context item before ingestion/assembly
    pub fn validate_item(item: &ContextItem) -> Result<(), ContextError> {
        // Invariant 1: FileContent, ToolResult, ModelOutput, CompactionSummary CANNOT assumed System role
        if item.role == ContextRole::System && item.source.is_data_plane() {
            return Err(ContextError::InvalidContextRole {
                source_name: item.source.as_str().to_string(),
                role: "system".to_string(),
            });
        }

        // Invariant 2: Untrusted content cannot masquerade as Control Plane
        if item.trust_level.is_untrusted() && item.source.is_control_plane() {
            return Err(ContextError::PolicyViolation {
                reason: format!(
                    "Untrusted content with trust level '{:?}' cannot use control-plane source '{:?}'",
                    item.trust_level, item.source
                ),
            });
        }

        Ok(())
    }

    /// Validates cross-session and cross-workspace isolation invariants
    pub fn validate_isolation(
        item: &ContextItem,
        target_workspace: Option<&WorkspaceId>,
        target_session: Option<&SessionId>,
    ) -> Result<(), ContextError> {
        // Workspace isolation
        if let (Some(item_ws), Some(target_ws)) = (&item.workspace_id, target_workspace) {
            if item_ws != target_ws {
                return Err(ContextError::IsolationViolation {
                    item_session: item_ws.as_str().to_string(),
                    target_session: target_ws.as_str().to_string(),
                });
            }
        }

        // Session isolation
        if let (Some(item_sess), Some(target_sess)) = (&item.session_id, target_session) {
            if item_sess != target_sess {
                return Err(ContextError::IsolationViolation {
                    item_session: item_sess.as_str().to_string(),
                    target_session: target_sess.as_str().to_string(),
                });
            }
        }

        Ok(())
    }

    /// Classification propagation check: derived classification CANNOT be lower than source classification
    pub fn validate_classification_propagation(
        source_classification: PrivacyClassification,
        derived_classification: PrivacyClassification,
    ) -> Result<(), ContextError> {
        if derived_classification < source_classification {
            return Err(ContextError::ClassificationConflict {
                source_classification: source_classification.to_string(),
                target_classification: derived_classification.to_string(),
            });
        }
        Ok(())
    }
}
