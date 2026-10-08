use crate::filesystem::errors::FilesystemError;
use crate::filesystem::types::FilesystemLimits;
use std::path::Path;

pub struct LimitsChecker;

impl LimitsChecker {
    pub fn check_read_limit(
        size_bytes: u64,
        limits: &FilesystemLimits,
        path: &Path,
    ) -> Result<(), FilesystemError> {
        if size_bytes > limits.max_read_bytes {
            return Err(FilesystemError::FileTooLarge {
                path: path.to_string_lossy().to_string(),
                size_bytes,
                limit_bytes: limits.max_read_bytes,
            });
        }
        Ok(())
    }

    pub fn check_write_limit(
        size_bytes: u64,
        limits: &FilesystemLimits,
        path: &Path,
    ) -> Result<(), FilesystemError> {
        if size_bytes > limits.max_write_bytes {
            return Err(FilesystemError::FileTooLarge {
                path: path.to_string_lossy().to_string(),
                size_bytes,
                limit_bytes: limits.max_write_bytes,
            });
        }
        Ok(())
    }

    pub fn check_copy_limit(
        size_bytes: u64,
        limits: &FilesystemLimits,
        path: &Path,
    ) -> Result<(), FilesystemError> {
        if size_bytes > limits.max_copy_size {
            return Err(FilesystemError::FileTooLarge {
                path: path.to_string_lossy().to_string(),
                size_bytes,
                limit_bytes: limits.max_copy_size,
            });
        }
        Ok(())
    }

    pub fn check_depth(
        current_depth: usize,
        limits: &FilesystemLimits,
    ) -> Result<(), FilesystemError> {
        if current_depth > limits.max_recursive_depth {
            return Err(FilesystemError::OperationBlocked {
                operation: "recursive_traversal".to_string(),
                reason: format!(
                    "Maximum recursive depth limit ({}) exceeded",
                    limits.max_recursive_depth
                ),
            });
        }
        Ok(())
    }
}
