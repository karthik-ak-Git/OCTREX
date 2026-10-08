use crate::context::types::{
    ContextInclusionReason, ContextRole, ContextSource, ContextTrustLevel, TokenCountKind,
};
use crate::ids::{SessionId, TaskId, WorkspaceId};
use crate::privacy::PrivacyClassification;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ContextItemId(pub String);

impl ContextItemId {
    pub fn new() -> Self {
        Self(format!("ctx-{}", uuid::Uuid::new_v4().simple()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for ContextItemId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for ContextItemId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Core ContextItem abstraction representing a discrete piece of contextual information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextItem {
    pub id: ContextItemId,
    pub source: ContextSource,
    pub source_id: Option<String>,
    pub role: ContextRole,
    pub content: String,
    pub trust_level: ContextTrustLevel,
    pub classification: PrivacyClassification,
    pub priority: u8, // 0 = P0 (Mandatory), 25 = P1, 50 = P2, 75 = P3, 100 = P4
    pub token_count: Option<usize>,
    pub token_count_kind: TokenCountKind,
    pub created_at: u64,
    pub expires_at: Option<u64>,
    pub provenance: Vec<String>,
    pub inclusion_reason: Option<ContextInclusionReason>,
    pub workspace_id: Option<WorkspaceId>,
    pub session_id: Option<SessionId>,
    pub task_id: Option<TaskId>,
    pub metadata: HashMap<String, String>,
}

impl ContextItem {
    pub fn new(source: ContextSource, role: ContextRole, content: impl Into<String>) -> Self {
        let trust_level = source.default_trust();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        let priority = if source.is_control_plane() { 0 } else { 50 };

        Self {
            id: ContextItemId::new(),
            source,
            source_id: None,
            role,
            content: content.into(),
            trust_level,
            classification: PrivacyClassification::Public,
            priority,
            token_count: None,
            token_count_kind: TokenCountKind::Unknown,
            created_at: now,
            expires_at: None,
            provenance: Vec::new(),
            inclusion_reason: None,
            workspace_id: None,
            session_id: None,
            task_id: None,
            metadata: HashMap::new(),
        }
    }

    pub fn with_source_id(mut self, source_id: impl Into<String>) -> Self {
        self.source_id = Some(source_id.into());
        self
    }

    pub fn with_classification(mut self, classification: PrivacyClassification) -> Self {
        self.classification = classification;
        self
    }

    pub fn with_trust_level(mut self, trust_level: ContextTrustLevel) -> Self {
        self.trust_level = trust_level;
        self
    }

    pub fn with_priority(mut self, priority: u8) -> Self {
        self.priority = priority;
        self
    }

    pub fn with_workspace(mut self, workspace_id: Option<WorkspaceId>) -> Self {
        self.workspace_id = workspace_id;
        self
    }

    pub fn with_session(mut self, session_id: Option<SessionId>) -> Self {
        self.session_id = session_id;
        self
    }

    pub fn with_task(mut self, task_id: Option<TaskId>) -> Self {
        self.task_id = task_id;
        self
    }

    pub fn with_provenance(mut self, provenance: impl Into<String>) -> Self {
        self.provenance.push(provenance.into());
        self
    }

    pub fn is_mandatory(&self) -> bool {
        self.priority == 0 || self.source.is_control_plane()
    }

    pub fn is_expired(&self) -> bool {
        if let Some(exp) = self.expires_at {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64;
            now > exp
        } else {
            false
        }
    }

    /// Safe summary for UI/diagnostics that NEVER leaks SECRET or RESTRICTED raw content
    pub fn safe_summary(&self) -> String {
        if self.classification >= PrivacyClassification::Secret {
            format!(
                "[REDACTED: {} item (id={}), {} tokens]",
                self.source.as_str(),
                self.id.as_str(),
                self.token_count.unwrap_or(0)
            )
        } else if self.classification >= PrivacyClassification::Restricted {
            format!(
                "[RESTRICTED {}: {} tokens]",
                self.source.as_str(),
                self.token_count.unwrap_or(0)
            )
        } else {
            let preview = if self.content.len() > 80 {
                format!("{}...", &self.content[..80])
            } else {
                self.content.clone()
            };
            format!("[{}: {}]", self.source.as_str(), preview.replace('\n', " "))
        }
    }
}
