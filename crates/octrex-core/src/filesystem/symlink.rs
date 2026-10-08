use crate::filesystem::errors::FilesystemError;
use std::fs;
use std::path::{Path, PathBuf};

pub struct SymlinkValidator;

impl SymlinkValidator {
    /// Validates that target_path (whether existing or to be created) does not escape trusted root
    /// via symlinks, junctions, or reparse points.
    pub fn validate_symlinks_and_reparse(
        root: &Path,
        target_path: &Path,
        is_create: bool,
    ) -> Result<(), FilesystemError> {
        // 1. Canonicalize trusted root
        let canonical_root = match fs::canonicalize(root) {
            Ok(c) => c,
            Err(e) => {
                return Err(FilesystemError::OutsideWorkspaceBoundary {
                    path: root.to_string_lossy().to_string(),
                    root: format!("Failed to canonicalize trusted root: {}", e),
                });
            }
        };

        // 2. Check if target_path exists
        if target_path.exists() {
            // Existing target must canonicalize safely
            let canonical_target = match fs::canonicalize(target_path) {
                Ok(c) => c,
                Err(_) => {
                    return Err(FilesystemError::SymlinkEscapeDetected {
                        path: target_path.to_string_lossy().to_string(),
                        target: "Broken symlink or unresolvable path target".to_string(),
                    });
                }
            };

            // Verify canonical target is inside canonical root
            if !canonical_target.starts_with(&canonical_root) {
                return Err(FilesystemError::SymlinkEscapeDetected {
                    path: target_path.to_string_lossy().to_string(),
                    target: canonical_target.to_string_lossy().to_string(),
                });
            }

            // Also inspect all intermediate path components for symlink / reparse point escapes
            Self::check_intermediate_components(&canonical_root, target_path)?;
        } else if is_create {
            // Target does not exist yet. Check its parent directory!
            let parent = match target_path.parent() {
                Some(p) => p,
                None => {
                    return Err(FilesystemError::OutsideWorkspaceBoundary {
                        path: target_path.to_string_lossy().to_string(),
                        root: canonical_root.to_string_lossy().to_string(),
                    });
                }
            };

            // If parent exists, canonicalize parent
            if parent.exists() {
                let canonical_parent = match fs::canonicalize(parent) {
                    Ok(c) => c,
                    Err(e) => {
                        return Err(FilesystemError::OutsideWorkspaceBoundary {
                            path: parent.to_string_lossy().to_string(),
                            root: format!("Failed to canonicalize parent: {}", e),
                        });
                    }
                };

                if !canonical_parent.starts_with(&canonical_root) {
                    return Err(FilesystemError::SymlinkEscapeDetected {
                        path: target_path.to_string_lossy().to_string(),
                        target: canonical_parent.to_string_lossy().to_string(),
                    });
                }

                Self::check_intermediate_components(&canonical_root, parent)?;
            } else {
                // If parent doesn't exist either, check ancestor components up to root
                Self::check_intermediate_components(&canonical_root, parent)?;
            }
        } else {
            return Err(FilesystemError::InvalidPath {
                path: target_path.to_string_lossy().to_string(),
            });
        }

        Ok(())
    }

    /// Check each component along path for symlinks or junctions that point outside canonical_root.
    fn check_intermediate_components(
        canonical_root: &Path,
        path: &Path,
    ) -> Result<(), FilesystemError> {
        let mut current = PathBuf::new();
        for component in path.components() {
            current.push(component);
            if current.exists() {
                let meta = match fs::symlink_metadata(&current) {
                    Ok(m) => m,
                    Err(_) => continue,
                };

                // Check if component is a symlink
                if meta.file_type().is_symlink() {
                    let canon_component = match fs::canonicalize(&current) {
                        Ok(c) => c,
                        Err(_) => {
                            return Err(FilesystemError::SymlinkEscapeDetected {
                                path: current.to_string_lossy().to_string(),
                                target: "Unresolvable symlink component".to_string(),
                            });
                        }
                    };

                    if !canon_component.starts_with(canonical_root) {
                        return Err(FilesystemError::SymlinkEscapeDetected {
                            path: current.to_string_lossy().to_string(),
                            target: canon_component.to_string_lossy().to_string(),
                        });
                    }
                }

                // Check Windows reparse points / junctions
                #[cfg(windows)]
                {
                    use std::os::windows::fs::MetadataExt;
                    let file_attributes = meta.file_attributes();
                    // FILE_ATTRIBUTE_REPARSE_POINT is 0x400
                    if (file_attributes & 0x400) != 0 {
                        let canon_component = match fs::canonicalize(&current) {
                            Ok(c) => c,
                            Err(_) => {
                                return Err(FilesystemError::ReparseOrJunctionEscape {
                                    path: current.to_string_lossy().to_string(),
                                });
                            }
                        };
                        if !canon_component.starts_with(canonical_root) {
                            return Err(FilesystemError::ReparseOrJunctionEscape {
                                path: current.to_string_lossy().to_string(),
                            });
                        }
                    }
                }
            }
        }
        Ok(())
    }
}
