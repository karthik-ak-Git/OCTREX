use crate::filesystem::errors::FilesystemError;
use crate::filesystem::types::FileCategory;
use std::fs::File;
use std::io::Read;
use std::path::Path;

pub struct FileValidator;

impl FileValidator {
    /// Inspect metadata to ensure node is not a FIFO, socket, device, or prohibited special file.
    pub fn validate_node_type(path: &Path) -> Result<(), FilesystemError> {
        if !path.exists() {
            return Ok(());
        }

        #[cfg(unix)]
        let file_type = meta.file_type();

        #[cfg(unix)]
        {
            use std::os::unix::fs::FileTypeExt;
            if file_type.is_fifo() {
                return Err(FilesystemError::SpecialNodeBlocked {
                    path: path.to_string_lossy().to_string(),
                    node_type: "FIFO / Named Pipe".to_string(),
                });
            }
            if file_type.is_socket() {
                return Err(FilesystemError::SpecialNodeBlocked {
                    path: path.to_string_lossy().to_string(),
                    node_type: "Unix Domain Socket".to_string(),
                });
            }
            if file_type.is_char_device() {
                return Err(FilesystemError::SpecialNodeBlocked {
                    path: path.to_string_lossy().to_string(),
                    node_type: "Character Device".to_string(),
                });
            }
            if file_type.is_block_device() {
                return Err(FilesystemError::SpecialNodeBlocked {
                    path: path.to_string_lossy().to_string(),
                    node_type: "Block Device".to_string(),
                });
            }
        }

        Ok(())
    }

    /// Detect if file content is text vs binary by scanning first 8192 bytes.
    pub fn detect_file_category(path: &Path) -> FileCategory {
        if !path.exists() || !path.is_file() {
            return FileCategory::Unknown;
        }

        let mut file = match File::open(path) {
            Ok(f) => f,
            Err(_) => return FileCategory::Unknown,
        };

        let mut buffer = [0u8; 8192];
        let bytes_read = match file.read(&mut buffer) {
            Ok(n) => n,
            Err(_) => return FileCategory::Unknown,
        };

        if bytes_read == 0 {
            return FileCategory::Text;
        }

        let sample = &buffer[..bytes_read];

        // If sample contains null byte 0x00, it's binary
        if sample.contains(&0) {
            return FileCategory::Binary;
        }

        // If sample is valid UTF-8, return Text; otherwise Binary
        if std::str::from_utf8(sample).is_ok() {
            FileCategory::Text
        } else {
            FileCategory::Binary
        }
    }
}
