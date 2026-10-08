use crate::ids::WorkspaceId;
use crate::privacy::PrivacyClassification;
use serde::{Deserialize, Serialize};
use std::fmt;

pub use crate::workspace::FileEntry;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FilesystemOperation {
    Read,
    Write,
    Create,
    Delete,
    Rename,
    Move,
    List,
    Stat,
    CreateDirectory,
    DeleteDirectory,
    Copy,
    Export,
    Import,
    RecursiveDelete,
}

impl fmt::Display for FilesystemOperation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FilesystemOperation::Read => write!(f, "read"),
            FilesystemOperation::Write => write!(f, "write"),
            FilesystemOperation::Create => write!(f, "create"),
            FilesystemOperation::Delete => write!(f, "delete"),
            FilesystemOperation::Rename => write!(f, "rename"),
            FilesystemOperation::Move => write!(f, "move"),
            FilesystemOperation::List => write!(f, "list"),
            FilesystemOperation::Stat => write!(f, "stat"),
            FilesystemOperation::CreateDirectory => write!(f, "create_directory"),
            FilesystemOperation::DeleteDirectory => write!(f, "delete_directory"),
            FilesystemOperation::Copy => write!(f, "copy"),
            FilesystemOperation::Export => write!(f, "export"),
            FilesystemOperation::Import => write!(f, "import"),
            FilesystemOperation::RecursiveDelete => write!(f, "recursive_delete"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OperationRisk {
    Low,
    Medium,
    High,
    Critical,
}

impl FilesystemOperation {
    pub fn default_risk(&self) -> OperationRisk {
        match self {
            FilesystemOperation::Stat | FilesystemOperation::List => OperationRisk::Low,
            FilesystemOperation::Read
            | FilesystemOperation::CreateDirectory
            | FilesystemOperation::Create
            | FilesystemOperation::Import => OperationRisk::Medium,
            FilesystemOperation::Write
            | FilesystemOperation::Rename
            | FilesystemOperation::Move
            | FilesystemOperation::Copy => OperationRisk::High,
            FilesystemOperation::Delete
            | FilesystemOperation::DeleteDirectory
            | FilesystemOperation::RecursiveDelete
            | FilesystemOperation::Export => OperationRisk::Critical,
        }
    }

    pub fn is_write_like(&self) -> bool {
        matches!(
            self,
            FilesystemOperation::Write
                | FilesystemOperation::Create
                | FilesystemOperation::Delete
                | FilesystemOperation::Rename
                | FilesystemOperation::Move
                | FilesystemOperation::CreateDirectory
                | FilesystemOperation::DeleteDirectory
                | FilesystemOperation::RecursiveDelete
                | FilesystemOperation::Import
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FilesystemDisposition {
    Allow,
    Block,
    RequireConfirmation,
    Unknown,
}

impl FilesystemDisposition {
    pub fn is_allowed(&self) -> bool {
        matches!(self, FilesystemDisposition::Allow)
    }

    /// INVARIANT 2: Unknown filesystem authorization MUST result in Block at enforcement time.
    pub fn sanitize(&self) -> Self {
        match self {
            FilesystemDisposition::Allow => FilesystemDisposition::Allow,
            FilesystemDisposition::RequireConfirmation => {
                FilesystemDisposition::RequireConfirmation
            }
            FilesystemDisposition::Block | FilesystemDisposition::Unknown => {
                FilesystemDisposition::Block
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProtectedPathAction {
    Block,
    ReadOnly,
    RequireConfirmation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtectedPath {
    pub pattern: String,
    pub action: ProtectedPathAction,
    pub description: String,
}

impl ProtectedPath {
    pub fn new(
        pattern: impl Into<String>,
        action: ProtectedPathAction,
        description: impl Into<String>,
    ) -> Self {
        Self {
            pattern: pattern.into(),
            action,
            description: description.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceSecurityPolicy {
    pub read_only: bool,
    pub allow_delete: bool,
    pub allow_recursive_delete: bool,
    pub allow_export: bool,
    pub allow_import: bool,
    pub max_read_bytes: u64,
    pub max_write_bytes: u64,
    pub protected_paths: Vec<ProtectedPath>,
}

impl Default for WorkspaceSecurityPolicy {
    fn default() -> Self {
        Self {
            read_only: false,
            allow_delete: true,
            allow_recursive_delete: false,
            allow_export: true,
            allow_import: true,
            max_read_bytes: 10 * 1024 * 1024,  // 10 MB default
            max_write_bytes: 10 * 1024 * 1024, // 10 MB default
            protected_paths: vec![
                ProtectedPath::new(
                    ".env",
                    ProtectedPathAction::Block,
                    "Environment file with secrets",
                ),
                ProtectedPath::new(
                    ".env.*",
                    ProtectedPathAction::Block,
                    "Environment configuration file",
                ),
                ProtectedPath::new(
                    "*.pem",
                    ProtectedPathAction::Block,
                    "PEM private key or certificate",
                ),
                ProtectedPath::new("*.key", ProtectedPathAction::Block, "Private key file"),
                ProtectedPath::new("id_rsa*", ProtectedPathAction::Block, "SSH private key"),
                ProtectedPath::new(
                    "id_ed25519*",
                    ProtectedPathAction::Block,
                    "SSH Ed25519 private key",
                ),
                ProtectedPath::new(
                    ".git/config",
                    ProtectedPathAction::ReadOnly,
                    "Git configuration",
                ),
                ProtectedPath::new(
                    "credentials*",
                    ProtectedPathAction::Block,
                    "Credentials file",
                ),
                ProtectedPath::new("secrets*", ProtectedPathAction::Block, "Secrets file"),
                ProtectedPath::new(
                    "*.pfx",
                    ProtectedPathAction::Block,
                    "PKCS#12 certificate container",
                ),
                ProtectedPath::new(
                    "*.p12",
                    ProtectedPathAction::Block,
                    "PKCS#12 certificate container",
                ),
            ],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilesystemLimits {
    pub max_read_bytes: u64,
    pub max_write_bytes: u64,
    pub max_file_size: u64,
    pub max_directory_entries: usize,
    pub max_recursive_depth: usize,
    pub max_copy_size: u64,
}

impl Default for FilesystemLimits {
    fn default() -> Self {
        Self {
            max_read_bytes: 10 * 1024 * 1024,  // 10 MB
            max_write_bytes: 10 * 1024 * 1024, // 10 MB
            max_file_size: 50 * 1024 * 1024,   // 50 MB
            max_directory_entries: 5000,
            max_recursive_depth: 10,
            max_copy_size: 50 * 1024 * 1024, // 50 MB
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilesystemDecision {
    pub decision_id: String,
    pub timestamp: u64,
    pub decision: FilesystemDisposition,
    pub operation: FilesystemOperation,
    pub workspace_id: Option<WorkspaceId>,
    pub requested_path: String,
    pub resolved_path: Option<String>,
    pub reason: String,
    pub matched_policy: Option<String>,
    pub policy_source: crate::privacy::PolicySource,
    pub classification: PrivacyClassification,
    pub risk_level: OperationRisk,
    pub warnings: Vec<String>,
    pub requires_confirmation: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceSecurityStatus {
    pub workspace_id: WorkspaceId,
    pub name: String,
    pub root_path: String,
    pub classification: PrivacyClassification,
    pub read_only: bool,
    pub security_status: String,
    pub protected_paths_count: usize,
    pub active_policy: WorkspaceSecurityPolicy,
    pub limits: FilesystemLimits,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FileCategory {
    Text,
    Binary,
    SpecialNode,
    Unknown,
}
