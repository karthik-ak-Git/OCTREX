use crate::filesystem::errors::FilesystemError;
use crate::filesystem::path::PathValidator;
use crate::filesystem::symlink::SymlinkValidator;
use std::path::{Path, PathBuf};

pub struct ArtifactSecurityBoundary;

impl ArtifactSecurityBoundary {
    /// Validate artifact relative path within workspace root or designated artifact root.
    pub fn validate_artifact_path(root: &Path, rel_path: &str) -> Result<PathBuf, FilesystemError> {
        let norm_rel = PathValidator::normalize_relative_path(rel_path)?;
        let resolved = PathValidator::resolve_workspace_path(root, rel_path)?;

        // Ensure artifact filename is not empty or traversal
        if norm_rel.as_os_str().is_empty() || norm_rel == Path::new(".") {
            return Err(FilesystemError::InvalidPath {
                path: rel_path.to_string(),
            });
        }

        // Validate symlink safety
        SymlinkValidator::validate_symlinks_and_reparse(root, &resolved, true)?;

        Ok(resolved)
    }
}
