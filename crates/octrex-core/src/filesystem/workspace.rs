use crate::filesystem::errors::FilesystemError;
use crate::filesystem::symlink::SymlinkValidator;
use crate::filesystem::validator::FileValidator;
use std::fs;
use std::path::{Path, PathBuf};

pub struct WorkspaceSecurityValidator;

impl WorkspaceSecurityValidator {
    /// Validates a potential workspace root path before registration.
    /// Ensures root exists, is accessible, is a directory, is not a system prohibited path,
    /// and is not a dangerous special node.
    pub fn validate_workspace_root(root_path: &Path) -> Result<PathBuf, FilesystemError> {
        if !root_path.exists() {
            return Err(FilesystemError::InvalidPath {
                path: format!(
                    "Workspace root path '{}' does not exist",
                    root_path.display()
                ),
            });
        }

        if !root_path.is_dir() {
            return Err(FilesystemError::InvalidPath {
                path: format!(
                    "Workspace root path '{}' is not a directory",
                    root_path.display()
                ),
            });
        }

        // Validate node type (ensure not FIFO, socket, or device)
        FileValidator::validate_node_type(root_path)?;

        // Canonicalize root path
        let canonical = match fs::canonicalize(root_path) {
            Ok(c) => c,
            Err(e) => {
                return Err(FilesystemError::IOError {
                    message: format!(
                        "Failed to canonicalize workspace root '{}': {}",
                        root_path.display(),
                        e
                    ),
                });
            }
        };

        // Prohibited system root paths check
        let canon_str = canonical.to_string_lossy().to_string();
        let clean_path = canon_str.replace('\\', "/").to_lowercase();

        let prohibited_roots = [
            "/",
            "/etc",
            "/sys",
            "/proc",
            "/dev",
            "/boot",
            "/usr",
            "c:/",
            "c:/windows",
            "c:/windows/system32",
            "c:/program files",
            "c:/program files (x86)",
        ];

        for prohibited in prohibited_roots {
            if clean_path == prohibited || clean_path == format!("{}/", prohibited) {
                return Err(FilesystemError::OutsideWorkspaceBoundary {
                    path: canon_str,
                    root: format!("Prohibited system root directory '{}'", prohibited),
                });
            }
        }

        // Validate symlink safety
        SymlinkValidator::validate_symlinks_and_reparse(&canonical, &canonical, false)?;

        Ok(canonical)
    }
}
