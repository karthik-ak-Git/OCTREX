use crate::db::repository::workspace::{SqliteWorkspaceRepository, WorkspaceRepository};
use crate::db::DatabaseManager;
use crate::error::OctrexError;
use crate::events::EventBus;
use crate::filesystem::audit::FilesystemAuditLogger;
use crate::filesystem::operations::SafeOperations;
use crate::filesystem::path::PathValidator;
use crate::filesystem::permissions::FilesystemPermissionEngine;
use crate::filesystem::symlink::SymlinkValidator;
use crate::filesystem::types::{
    FileEntry, FilesystemDecision, FilesystemDisposition, FilesystemLimits, FilesystemOperation,
    WorkspaceSecurityPolicy, WorkspaceSecurityStatus,
};
use crate::filesystem::workspace::WorkspaceSecurityValidator;
use crate::ids::WorkspaceId;
use crate::network::NetworkSecurityService;
use crate::privacy::{PrivacyClassification, PrivacyGate};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

pub struct FilesystemSecurityService {
    db: Arc<DatabaseManager>,
    event_bus: Arc<EventBus>,
    _privacy_gate: Arc<PrivacyGate>,
    _network_security: Arc<NetworkSecurityService>,
    workspace_repo: Arc<SqliteWorkspaceRepository>,
    policies: Arc<RwLock<HashMap<WorkspaceId, WorkspaceSecurityPolicy>>>,
    limits: FilesystemLimits,
    pending_confirmations: Arc<RwLock<HashMap<String, FilesystemDecision>>>,
}

impl FilesystemSecurityService {
    pub fn new(
        db: Arc<DatabaseManager>,
        event_bus: Arc<EventBus>,
        privacy_gate: Arc<PrivacyGate>,
        network_security: Arc<NetworkSecurityService>,
    ) -> Self {
        let workspace_repo = Arc::new(SqliteWorkspaceRepository::new((*db).clone()));
        Self {
            db,
            event_bus,
            _privacy_gate: privacy_gate,
            _network_security: network_security,
            workspace_repo,
            policies: Arc::new(RwLock::new(HashMap::new())),
            limits: FilesystemLimits::default(),
            pending_confirmations: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Evaluates a requested filesystem operation against backend trusted workspace state and security policy.
    pub fn evaluate_operation(
        &self,
        workspace_id: Option<&WorkspaceId>,
        root_override: Option<&Path>,
        rel_path: &str,
        operation: FilesystemOperation,
    ) -> Result<FilesystemDecision, OctrexError> {
        // 1. Resolve trusted workspace root from DB or override
        let (trusted_root, ws_classification) = if let Some(ws_id) = workspace_id {
            match self.workspace_repo.get_workspace(ws_id)? {
                Some(ws) => {
                    let class = ws
                        .classification
                        .parse()
                        .unwrap_or(PrivacyClassification::Public);
                    (ws.path, class)
                }
                None => {
                    return Err(OctrexError::Filesystem {
                        message: format!("Workspace '{}' not found in database", ws_id),
                    });
                }
            }
        } else if let Some(r) = root_override {
            (r.to_path_buf(), PrivacyClassification::Public)
        } else {
            (PathBuf::from("."), PrivacyClassification::Public)
        };

        // Validate trusted root
        let canonical_root = WorkspaceSecurityValidator::validate_workspace_root(&trusted_root)
            .map_err(|e| OctrexError::Filesystem {
                message: e.to_string(),
            })?;

        // 2. Get active policy for workspace
        let policy = if let Some(ws_id) = workspace_id {
            let guard = self.policies.read().unwrap();
            guard.get(ws_id).cloned().unwrap_or_default()
        } else {
            WorkspaceSecurityPolicy::default()
        };

        // 3. Evaluate permission decision
        let mut decision = FilesystemPermissionEngine::evaluate(
            workspace_id.cloned(),
            rel_path,
            operation,
            &policy,
            ws_classification,
        );

        // 4. Validate path resolution & symlinks
        match PathValidator::resolve_workspace_path(&canonical_root, rel_path) {
            Ok(target_path) => {
                let is_create = matches!(
                    operation,
                    FilesystemOperation::Write
                        | FilesystemOperation::Create
                        | FilesystemOperation::CreateDirectory
                        | FilesystemOperation::Copy
                        | FilesystemOperation::Import
                );

                match SymlinkValidator::validate_symlinks_and_reparse(
                    &canonical_root,
                    &target_path,
                    is_create,
                ) {
                    Ok(_) => {
                        decision.resolved_path = Some(target_path.to_string_lossy().to_string());
                    }
                    Err(symlink_err) => {
                        decision.decision = FilesystemDisposition::Block;
                        decision.reason = symlink_err.to_string();
                        decision
                            .warnings
                            .push("Symlink/Reparse escape attempt blocked".to_string());
                    }
                }
            }
            Err(path_err) => {
                decision.decision = FilesystemDisposition::Block;
                decision.reason = path_err.to_string();
                decision
                    .warnings
                    .push("Path traversal attempt blocked".to_string());
            }
        }

        // INVARIANT 2: Unknown filesystem authorization MUST result in Block
        decision.decision = decision.decision.sanitize();

        // Audit log decision
        FilesystemAuditLogger::log_decision(&self.db, &self.event_bus, &decision);

        if decision.requires_confirmation {
            let mut pending = self.pending_confirmations.write().unwrap();
            pending.insert(decision.decision_id.clone(), decision.clone());
        }

        Ok(decision)
    }

    /// Safely read text file from workspace.
    pub fn read_file(
        &self,
        workspace_id: &WorkspaceId,
        rel_path: &str,
    ) -> Result<String, OctrexError> {
        let decision = self.evaluate_operation(
            Some(workspace_id),
            None,
            rel_path,
            FilesystemOperation::Read,
        )?;

        if !decision.decision.is_allowed() {
            return Err(OctrexError::PermissionDenied {
                reason: format!("{}: {}", rel_path, decision.reason),
            });
        }

        let target_path = PathBuf::from(decision.resolved_path.as_deref().ok_or_else(|| {
            OctrexError::Filesystem {
                message: "Resolved path missing".to_string(),
            }
        })?);

        SafeOperations::safe_read_text(&target_path, &self.limits).map_err(|e| {
            OctrexError::Filesystem {
                message: e.to_string(),
            }
        })
    }

    /// Safely read binary file from workspace.
    pub fn read_file_bytes(
        &self,
        workspace_id: &WorkspaceId,
        rel_path: &str,
    ) -> Result<Vec<u8>, OctrexError> {
        let decision = self.evaluate_operation(
            Some(workspace_id),
            None,
            rel_path,
            FilesystemOperation::Read,
        )?;

        if !decision.decision.is_allowed() {
            return Err(OctrexError::PermissionDenied {
                reason: format!("{}: {}", rel_path, decision.reason),
            });
        }

        let target_path = PathBuf::from(decision.resolved_path.as_deref().ok_or_else(|| {
            OctrexError::Filesystem {
                message: "Resolved path missing".to_string(),
            }
        })?);

        SafeOperations::safe_read_bytes(&target_path, &self.limits).map_err(|e| {
            OctrexError::Filesystem {
                message: e.to_string(),
            }
        })
    }

    /// Safely write text file to workspace.
    pub fn write_file(
        &self,
        workspace_id: &WorkspaceId,
        rel_path: &str,
        content: &str,
    ) -> Result<usize, OctrexError> {
        self.write_file_bytes(workspace_id, rel_path, content.as_bytes())
    }

    /// Safely write binary file to workspace.
    pub fn write_file_bytes(
        &self,
        workspace_id: &WorkspaceId,
        rel_path: &str,
        content: &[u8],
    ) -> Result<usize, OctrexError> {
        let decision = self.evaluate_operation(
            Some(workspace_id),
            None,
            rel_path,
            FilesystemOperation::Write,
        )?;

        if !decision.decision.is_allowed() {
            return Err(OctrexError::PermissionDenied {
                reason: format!("{}: {}", rel_path, decision.reason),
            });
        }

        let target_path = PathBuf::from(decision.resolved_path.as_deref().ok_or_else(|| {
            OctrexError::Filesystem {
                message: "Resolved path missing".to_string(),
            }
        })?);

        SafeOperations::safe_write_bytes(&target_path, content, &self.limits).map_err(|e| {
            OctrexError::Filesystem {
                message: e.to_string(),
            }
        })
    }

    /// Safely delete file or directory.
    pub fn delete_file(
        &self,
        workspace_id: &WorkspaceId,
        rel_path: &str,
        recursive: bool,
    ) -> Result<(), OctrexError> {
        let op = if recursive {
            FilesystemOperation::RecursiveDelete
        } else {
            FilesystemOperation::Delete
        };

        let decision = self.evaluate_operation(Some(workspace_id), None, rel_path, op)?;

        if !decision.decision.is_allowed() {
            return Err(OctrexError::PermissionDenied {
                reason: format!("{}: {}", rel_path, decision.reason),
            });
        }

        let target_path = PathBuf::from(decision.resolved_path.as_deref().ok_or_else(|| {
            OctrexError::Filesystem {
                message: "Resolved path missing".to_string(),
            }
        })?);

        SafeOperations::safe_delete(&target_path, recursive).map_err(|e| OctrexError::Filesystem {
            message: e.to_string(),
        })
    }

    /// Safely list workspace directory entries.
    pub fn list_directory(
        &self,
        workspace_id: &WorkspaceId,
        rel_path: &str,
    ) -> Result<Vec<FileEntry>, OctrexError> {
        let decision = self.evaluate_operation(
            Some(workspace_id),
            None,
            rel_path,
            FilesystemOperation::List,
        )?;

        if !decision.decision.is_allowed() {
            return Err(OctrexError::PermissionDenied {
                reason: format!("{}: {}", rel_path, decision.reason),
            });
        }

        let ws = self
            .workspace_repo
            .get_workspace(workspace_id)?
            .ok_or_else(|| OctrexError::Filesystem {
                message: format!("Workspace '{}' not found", workspace_id),
            })?;

        let target_path = PathBuf::from(decision.resolved_path.as_deref().ok_or_else(|| {
            OctrexError::Filesystem {
                message: "Resolved path missing".to_string(),
            }
        })?);

        SafeOperations::safe_list_directory(&ws.path, &target_path, &self.limits).map_err(|e| {
            OctrexError::Filesystem {
                message: e.to_string(),
            }
        })
    }

    /// Safely move/rename file between workspace paths.
    pub fn move_file(
        &self,
        src_workspace_id: &WorkspaceId,
        src_rel_path: &str,
        dest_workspace_id: &WorkspaceId,
        dest_rel_path: &str,
    ) -> Result<(), OctrexError> {
        // Cross-workspace moves blocked by default (Section 26)
        if src_workspace_id != dest_workspace_id {
            return Err(OctrexError::Filesystem {
                message: "Cross-workspace move operations are blocked by default".to_string(),
            });
        }

        let src_decision = self.evaluate_operation(
            Some(src_workspace_id),
            None,
            src_rel_path,
            FilesystemOperation::Move,
        )?;
        let dest_decision = self.evaluate_operation(
            Some(dest_workspace_id),
            None,
            dest_rel_path,
            FilesystemOperation::Write,
        )?;

        if !src_decision.decision.is_allowed() || !dest_decision.decision.is_allowed() {
            return Err(OctrexError::PermissionDenied {
                reason: format!(
                    "{} -> {}: Move operation denied by security boundary",
                    src_rel_path, dest_rel_path
                ),
            });
        }

        let src_path = PathBuf::from(src_decision.resolved_path.unwrap());
        let dest_path = PathBuf::from(dest_decision.resolved_path.unwrap());

        SafeOperations::safe_move(&src_path, &dest_path).map_err(|e| OctrexError::Filesystem {
            message: e.to_string(),
        })
    }

    /// Safely copy file between workspace paths.
    pub fn copy_file(
        &self,
        src_workspace_id: &WorkspaceId,
        src_rel_path: &str,
        dest_workspace_id: &WorkspaceId,
        dest_rel_path: &str,
    ) -> Result<u64, OctrexError> {
        let src_decision = self.evaluate_operation(
            Some(src_workspace_id),
            None,
            src_rel_path,
            FilesystemOperation::Read,
        )?;
        let dest_decision = self.evaluate_operation(
            Some(dest_workspace_id),
            None,
            dest_rel_path,
            FilesystemOperation::Copy,
        )?;

        if !src_decision.decision.is_allowed() || !dest_decision.decision.is_allowed() {
            return Err(OctrexError::PermissionDenied {
                reason: format!(
                    "{} -> {}: Copy operation denied by security boundary",
                    src_rel_path, dest_rel_path
                ),
            });
        }

        let src_path = PathBuf::from(src_decision.resolved_path.unwrap());
        let dest_path = PathBuf::from(dest_decision.resolved_path.unwrap());

        SafeOperations::safe_copy(&src_path, &dest_path, &self.limits).map_err(|e| {
            OctrexError::Filesystem {
                message: e.to_string(),
            }
        })
    }

    /// Export file outside workspace (requires Filesystem + Phase 6 Privacy + Phase 7 Network security check).
    pub fn export_file(
        &self,
        workspace_id: &WorkspaceId,
        rel_path: &str,
        dest_external_path: &str,
    ) -> Result<(), OctrexError> {
        let decision = self.evaluate_operation(
            Some(workspace_id),
            None,
            rel_path,
            FilesystemOperation::Export,
        )?;

        if !decision.decision.is_allowed() {
            return Err(OctrexError::PermissionDenied {
                reason: format!("{}: {}", rel_path, decision.reason),
            });
        }

        // Check Phase 6 Privacy: Confidential/Secret data cannot be exported without authorization
        if decision.classification == PrivacyClassification::Secret
            || decision.classification == PrivacyClassification::Restricted
        {
            return Err(OctrexError::PrivacyBlocked {
                reason: format!(
                    "File '{}' is classified as {:?}; external export prohibited",
                    rel_path, decision.classification
                ),
            });
        }

        let src_path = PathBuf::from(decision.resolved_path.unwrap());
        let dest_path = PathBuf::from(dest_external_path);

        SafeOperations::safe_copy(&src_path, &dest_path, &self.limits).map_err(|e| {
            OctrexError::Filesystem {
                message: e.to_string(),
            }
        })?;

        Ok(())
    }

    /// Confirm or deny a pending high-risk filesystem operation.
    pub fn confirm_operation(
        &self,
        decision_id: &str,
        granted: bool,
    ) -> Result<FilesystemDecision, OctrexError> {
        let mut pending = self.pending_confirmations.write().unwrap();
        let mut decision = pending
            .remove(decision_id)
            .ok_or_else(|| OctrexError::NotFound {
                resource: format!("Pending confirmation decision '{}'", decision_id),
            })?;

        decision.decision = if granted {
            FilesystemDisposition::Allow
        } else {
            FilesystemDisposition::Block
        };
        decision.requires_confirmation = false;
        decision.reason = if granted {
            "Operation explicitly confirmed by user".to_string()
        } else {
            "Operation denied during confirmation prompt".to_string()
        };

        FilesystemAuditLogger::log_decision(&self.db, &self.event_bus, &decision);

        Ok(decision)
    }

    /// Get security status for workspace.
    pub fn get_workspace_security(
        &self,
        workspace_id: &WorkspaceId,
    ) -> Result<WorkspaceSecurityStatus, OctrexError> {
        let ws = self
            .workspace_repo
            .get_workspace(workspace_id)?
            .ok_or_else(|| OctrexError::Filesystem {
                message: format!("Workspace '{}' not found", workspace_id),
            })?;

        let class = ws
            .classification
            .parse()
            .unwrap_or(PrivacyClassification::Public);
        let guard = self.policies.read().unwrap();
        let policy = guard.get(workspace_id).cloned().unwrap_or_default();

        let security_status = if policy.read_only {
            "READ_ONLY".to_string()
        } else if class == PrivacyClassification::Secret
            || class == PrivacyClassification::Restricted
        {
            "RESTRICTED".to_string()
        } else {
            "SECURE_BOUNDED".to_string()
        };

        Ok(WorkspaceSecurityStatus {
            workspace_id: ws.id,
            name: ws.name,
            root_path: ws.path.to_string_lossy().to_string(),
            classification: class,
            read_only: policy.read_only,
            security_status,
            protected_paths_count: policy.protected_paths.len(),
            active_policy: policy,
            limits: self.limits.clone(),
        })
    }

    /// Update workspace security policy.
    pub fn update_workspace_policy(
        &self,
        workspace_id: &WorkspaceId,
        policy: WorkspaceSecurityPolicy,
    ) -> Result<WorkspaceSecurityPolicy, OctrexError> {
        let _ws = self
            .workspace_repo
            .get_workspace(workspace_id)?
            .ok_or_else(|| OctrexError::Filesystem {
                message: format!("Workspace '{}' not found", workspace_id),
            })?;

        let mut guard = self.policies.write().unwrap();
        guard.insert(workspace_id.clone(), policy.clone());

        Ok(policy)
    }
}
