use crate::workspace::types::{FileEntry, WorkspaceInfo};
use std::fs;
use std::path::Path;

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

    /// List directory file tree up to max depth 3
    pub fn list_tree<P: AsRef<Path>>(path: P) -> Vec<FileEntry> {
        let p = path.as_ref();
        let mut entries = Vec::new();
        if p.exists() && p.is_dir() {
            Self::collect_entries_recursive(p, p, 0, 3, &mut entries);
        }
        entries
    }

    /// Read file content safely within specified workspace path
    pub fn read_file<P: AsRef<Path>>(base: P, rel_path: &str) -> anyhow::Result<String> {
        let full_path = base.as_ref().join(rel_path);
        if !full_path.exists() || !full_path.is_file() {
            anyhow::bail!("File '{}' does not exist or is not a file", rel_path);
        }
        let content = fs::read_to_string(&full_path)?;
        Ok(content)
    }

    /// Write/update file content safely within specified workspace path
    pub fn write_file<P: AsRef<Path>>(
        base: P,
        rel_path: &str,
        content: &str,
    ) -> anyhow::Result<usize> {
        let full_path = base.as_ref().join(rel_path);
        if let Some(parent) = full_path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)?;
            }
        }
        fs::write(&full_path, content)?;
        Ok(content.len())
    }

    fn collect_entries_recursive(
        root: &Path,
        dir: &Path,
        current_depth: usize,
        max_depth: usize,
        out: &mut Vec<FileEntry>,
    ) {
        if current_depth > max_depth {
            return;
        }

        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                let file_name = match path.file_name().and_then(|f| f.to_str()) {
                    Some(n) => n.to_string(),
                    None => continue,
                };

                if file_name.starts_with('.')
                    || file_name == "node_modules"
                    || file_name == "target"
                    || file_name == "out"
                {
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

                if is_dir {
                    Self::collect_entries_recursive(root, &path, current_depth + 1, max_depth, out);
                }
            }
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
                    if file_name.starts_with('.')
                        || file_name == "node_modules"
                        || file_name == "target"
                        || file_name == "out"
                    {
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
