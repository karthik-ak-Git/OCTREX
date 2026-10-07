use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceInfo {
    pub path: PathBuf,
    pub exists: bool,
    pub is_dir: bool,
    pub total_files: usize,
    pub name: String,
}

pub struct WorkspaceManager;

impl WorkspaceManager {
    /// Inspect an actual folder path on the local filesystem and return live metadata.
    pub fn inspect<P: AsRef<Path>>(path: P) -> WorkspaceInfo {
        let p = path.as_ref();
        let exists = p.exists();
        let is_dir = p.is_dir();
        
        let name = p
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Root")
            .to_string();

        let total_files = if exists && is_dir {
            Self::count_files_recursive(p, 0, 3)
        } else {
            0
        };

        WorkspaceInfo {
            path: p.to_path_buf(),
            exists,
            is_dir,
            total_files,
            name,
        }
    }

    fn count_files_recursive(dir: &Path, current_depth: usize, max_depth: usize) -> usize {
        if current_depth > max_depth {
            return 0;
        }

        let mut count = 0;
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(file_name) = path.file_name().and_then(|f| f.to_str()) {
                    if file_name.starts_with('.') || file_name == "node_modules" || file_name == "target" {
                        continue;
                    }
                }
                if path.is_file() {
                    count += 1;
                } else if path.is_dir() {
                    count += Self::count_files_recursive(&path, current_depth + 1, max_depth);
                }
            }
        }
        count
    }
}
