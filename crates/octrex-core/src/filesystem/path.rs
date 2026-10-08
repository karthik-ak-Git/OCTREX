use crate::filesystem::errors::FilesystemError;
use std::path::{Component, Path, PathBuf};

pub struct PathValidator;

impl PathValidator {
    /// Normalize relative path and reject any path traversal or invalid patterns.
    pub fn normalize_relative_path(rel_path: &str) -> Result<PathBuf, FilesystemError> {
        let raw = rel_path.trim();

        if raw.is_empty() {
            return Ok(PathBuf::from("."));
        }

        // Check null bytes or encoded null bytes
        if raw.contains('\0') || raw.contains("%00") {
            return Err(FilesystemError::PathTraversalDetected {
                path: raw.to_string(),
                reason: "Null byte detected in path string".to_string(),
            });
        }

        // Check UNC paths or NT device paths
        if raw.starts_with(r"\\")
            || raw.starts_with("//")
            || raw.starts_with(r"\\?\")
            || raw.starts_with(r"\\.\")
        {
            return Err(FilesystemError::PathTraversalDetected {
                path: raw.to_string(),
                reason: "UNC or NT device paths are prohibited".to_string(),
            });
        }

        // Check Windows drive letter prefixes (e.g. C:, D:\)
        if raw.len() >= 2 {
            let bytes = raw.as_bytes();
            if bytes[0].is_ascii_alphabetic() && bytes[1] == b':' {
                return Err(FilesystemError::PathTraversalDetected {
                    path: raw.to_string(),
                    reason: "Absolute drive letter path is not a relative path".to_string(),
                });
            }
        }

        // Standardize separators to inspect components
        let clean_str = raw.replace('\\', "/");

        // Reject explicit traversal strings
        if clean_str.contains("../")
            || clean_str.contains("/..")
            || clean_str == ".."
            || clean_str.contains("...")
        {
            return Err(FilesystemError::PathTraversalDetected {
                path: raw.to_string(),
                reason: "Path contains parent directory traversal ('..')".to_string(),
            });
        }

        let raw_path = Path::new(raw);
        let mut normalized = PathBuf::new();

        for comp in raw_path.components() {
            match comp {
                Component::Normal(segment) => {
                    let seg_str = segment.to_string_lossy();
                    if seg_str == ".." || seg_str == "..." {
                        return Err(FilesystemError::PathTraversalDetected {
                            path: raw.to_string(),
                            reason: "Component traversal segment detected".to_string(),
                        });
                    }
                    normalized.push(segment);
                }
                Component::CurDir => {
                    // Current directory '.' is okay to ignore or skip
                }
                Component::ParentDir => {
                    return Err(FilesystemError::PathTraversalDetected {
                        path: raw.to_string(),
                        reason: "ParentDir component detected".to_string(),
                    });
                }
                Component::RootDir | Component::Prefix(_) => {
                    return Err(FilesystemError::PathTraversalDetected {
                        path: raw.to_string(),
                        reason: "Absolute root or drive prefix in relative path".to_string(),
                    });
                }
            }
        }

        if normalized.as_os_str().is_empty() {
            Ok(PathBuf::from("."))
        } else {
            Ok(normalized)
        }
    }

    /// Resolve a normalized relative path against a trusted workspace root.
    /// Ensures the resulting path stays within the trusted root boundary.
    pub fn resolve_workspace_path(root: &Path, rel_path: &str) -> Result<PathBuf, FilesystemError> {
        let norm_rel = Self::normalize_relative_path(rel_path)?;

        let target = if norm_rel == Path::new(".") {
            root.to_path_buf()
        } else {
            root.join(norm_rel)
        };

        Ok(target)
    }
}
