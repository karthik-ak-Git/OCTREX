use crate::filesystem::errors::FilesystemError;
use crate::filesystem::limits::LimitsChecker;
use crate::filesystem::types::{FileEntry, FilesystemLimits};
use crate::filesystem::validator::FileValidator;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;

pub struct SafeOperations;

impl SafeOperations {
    /// Safe file read returning raw bytes.
    pub fn safe_read_bytes(
        target_path: &Path,
        limits: &FilesystemLimits,
    ) -> Result<Vec<u8>, FilesystemError> {
        FileValidator::validate_node_type(target_path)?;

        if !target_path.exists() || !target_path.is_file() {
            return Err(FilesystemError::InvalidPath {
                path: format!(
                    "File '{}' does not exist or is not a regular file",
                    target_path.display()
                ),
            });
        }

        let metadata = fs::metadata(target_path)?;
        LimitsChecker::check_read_limit(metadata.len(), limits, target_path)?;

        let mut file = File::open(target_path)?;
        let mut buffer = Vec::with_capacity(metadata.len() as usize);
        file.read_to_end(&mut buffer)?;

        Ok(buffer)
    }

    /// Safe file read returning UTF-8 string.
    pub fn safe_read_text(
        target_path: &Path,
        limits: &FilesystemLimits,
    ) -> Result<String, FilesystemError> {
        let bytes = Self::safe_read_bytes(target_path, limits)?;
        String::from_utf8(bytes).map_err(|_| FilesystemError::OperationBlocked {
            operation: "read_text".to_string(),
            reason: "File contains invalid UTF-8 non-text byte sequences".to_string(),
        })
    }

    /// Safe file write from byte payload.
    pub fn safe_write_bytes(
        target_path: &Path,
        content: &[u8],
        limits: &FilesystemLimits,
    ) -> Result<usize, FilesystemError> {
        LimitsChecker::check_write_limit(content.len() as u64, limits, target_path)?;

        if let Some(parent) = target_path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)?;
            }
        }

        let mut file = File::create(target_path)?;
        file.write_all(content)?;
        Ok(content.len())
    }

    /// Safe file deletion.
    pub fn safe_delete(target_path: &Path, recursive: bool) -> Result<(), FilesystemError> {
        if !target_path.exists() {
            return Err(FilesystemError::InvalidPath {
                path: format!("Path '{}' does not exist", target_path.display()),
            });
        }

        if target_path.is_dir() {
            if recursive {
                fs::remove_dir_all(target_path)?;
            } else {
                fs::remove_dir(target_path)?;
            }
        } else {
            fs::remove_file(target_path)?;
        }

        Ok(())
    }

    /// Safe directory listing bounded by depth and entry limits.
    pub fn safe_list_directory(
        root: &Path,
        dir_path: &Path,
        limits: &FilesystemLimits,
    ) -> Result<Vec<FileEntry>, FilesystemError> {
        if !dir_path.exists() || !dir_path.is_dir() {
            return Err(FilesystemError::InvalidPath {
                path: format!("Directory '{}' does not exist", dir_path.display()),
            });
        }

        let mut entries = Vec::new();
        Self::collect_entries(root, dir_path, 0, limits, &mut entries)?;
        Ok(entries)
    }

    fn collect_entries(
        root: &Path,
        current_dir: &Path,
        depth: usize,
        limits: &FilesystemLimits,
        out: &mut Vec<FileEntry>,
    ) -> Result<(), FilesystemError> {
        LimitsChecker::check_depth(depth, limits)?;

        let read_dir = fs::read_dir(current_dir)?;
        for entry_res in read_dir {
            if out.len() >= limits.max_directory_entries {
                break;
            }

            let entry = entry_res?;
            let path = entry.path();
            let file_name = match path.file_name().and_then(|n| n.to_str()) {
                Some(n) => n.to_string(),
                None => continue,
            };

            // Filter out system files like .git or target if deeply listing
            if file_name.starts_with('.') || file_name == "node_modules" || file_name == "target" {
                continue;
            }

            let is_dir = path.is_dir();
            let size_bytes = if is_dir {
                0
            } else {
                path.metadata().map(|m| m.len()).unwrap_or(0)
            };

            let rel_path = path
                .strip_prefix(root)
                .unwrap_or(&path)
                .to_string_lossy()
                .to_string();

            out.push(FileEntry {
                name: file_name,
                path: path.to_string_lossy().to_string(),
                rel_path,
                is_dir,
                size_bytes,
            });

            if is_dir && depth + 1 <= limits.max_recursive_depth {
                Self::collect_entries(root, &path, depth + 1, limits, out)?;
            }
        }

        Ok(())
    }

    /// Safe file copy (validating source and dest).
    pub fn safe_copy(
        src: &Path,
        dest: &Path,
        limits: &FilesystemLimits,
    ) -> Result<u64, FilesystemError> {
        FileValidator::validate_node_type(src)?;

        if !src.exists() || !src.is_file() {
            return Err(FilesystemError::InvalidPath {
                path: format!("Source file '{}' does not exist", src.display()),
            });
        }

        let meta = fs::metadata(src)?;
        LimitsChecker::check_copy_limit(meta.len(), limits, src)?;

        if let Some(parent) = dest.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)?;
            }
        }

        let bytes_copied = fs::copy(src, dest)?;
        Ok(bytes_copied)
    }

    /// Safe file move/rename (validating source and dest).
    pub fn safe_move(src: &Path, dest: &Path) -> Result<(), FilesystemError> {
        if !src.exists() {
            return Err(FilesystemError::InvalidPath {
                path: format!("Source path '{}' does not exist", src.display()),
            });
        }

        if let Some(parent) = dest.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)?;
            }
        }

        fs::rename(src, dest)?;
        Ok(())
    }
}
