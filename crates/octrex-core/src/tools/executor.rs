use crate::tools::errors::ToolError;
use crate::tools::sandbox::ToolExecutionContext;
use crate::tools::types::{ToolCapability, ToolId};
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};

pub struct BuiltInToolExecutor;

impl BuiltInToolExecutor {
    pub async fn execute(
        tool_id: &ToolId,
        args: &Value,
        ctx: &ToolExecutionContext,
    ) -> Result<Value, ToolError> {
        ctx.check_cancellation().map_err(ToolError::Cancelled)?;

        match tool_id.as_str() {
            "workspace_list" => Self::execute_workspace_list(args, ctx),
            "workspace_read" => Self::execute_workspace_read(args, ctx),
            "workspace_write" => Self::execute_workspace_write(args, ctx),
            "workspace_search" => Self::execute_workspace_search(args, ctx),
            "system_info" => Self::execute_system_info(args, ctx),
            other if other.starts_with("document.") => {
                crate::documents::tools::execute_document_tool(other, args, ctx)
            }
            other => Err(ToolError::ToolNotFound(other.to_string())),
        }
    }

    fn resolve_workspace_path(
        rel_path: &str,
        ctx: &ToolExecutionContext,
    ) -> Result<PathBuf, ToolError> {
        let ws_root = ctx.workspace_path.as_ref().ok_or_else(|| {
            ToolError::FilesystemDenied("No active workspace path bound to context".to_string())
        })?;

        let clean = rel_path.trim_start_matches('/').trim_start_matches('\\');
        let target = ws_root.join(clean);

        // Security check: Canonicalize or verify path is inside workspace root
        if let (Ok(canonical_root), Ok(canonical_target)) =
            (ws_root.canonicalize(), target.canonicalize())
        {
            if !canonical_target.starts_with(&canonical_root) {
                return Err(ToolError::FilesystemDenied(format!(
                    "Path '{}' escapes workspace boundary",
                    rel_path
                )));
            }
        } else if target.to_string_lossy().contains("..") {
            return Err(ToolError::FilesystemDenied(format!(
                "Path '{}' contains invalid parent directory references",
                rel_path
            )));
        }

        Ok(target)
    }

    fn execute_workspace_list(
        args: &Value,
        ctx: &ToolExecutionContext,
    ) -> Result<Value, ToolError> {
        if !ctx.has_capability(&ToolCapability::FilesystemList)
            && !ctx.has_capability(&ToolCapability::WorkspaceRead)
        {
            return Err(ToolError::CapabilityDenied {
                capability: "FilesystemList".to_string(),
                reason: "Missing list capability".to_string(),
            });
        }

        let subpath = args.get("subpath").and_then(|s| s.as_str()).unwrap_or("");

        let ws_root = ctx.workspace_path.as_ref().ok_or_else(|| {
            ToolError::FilesystemDenied("No active workspace path set".to_string())
        })?;

        let target_dir = if subpath.is_empty() {
            ws_root.clone()
        } else {
            Self::resolve_workspace_path(subpath, ctx)?
        };

        if !target_dir.exists() {
            return Err(ToolError::ToolExecutionFailed(format!(
                "Directory '{}' does not exist",
                target_dir.display()
            )));
        }

        let entries =
            fs::read_dir(&target_dir).map_err(|e| ToolError::ToolExecutionFailed(e.to_string()))?;

        let mut files = Vec::new();
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            let is_dir = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);
            files.push(serde_json::json!({
                "name": name,
                "is_dir": is_dir
            }));
        }

        Ok(serde_json::json!({ "files": files }))
    }

    fn execute_workspace_read(
        args: &Value,
        ctx: &ToolExecutionContext,
    ) -> Result<Value, ToolError> {
        if !ctx.has_capability(&ToolCapability::FilesystemRead)
            && !ctx.has_capability(&ToolCapability::WorkspaceRead)
        {
            return Err(ToolError::CapabilityDenied {
                capability: "FilesystemRead".to_string(),
                reason: "Missing read capability".to_string(),
            });
        }

        let path_str = args
            .get("path")
            .and_then(|p| p.as_str())
            .ok_or_else(|| ToolError::InvalidArguments("Missing 'path' argument".to_string()))?;

        let target_file = Self::resolve_workspace_path(path_str, ctx)?;

        if !target_file.exists() || !target_file.is_file() {
            return Err(ToolError::ToolExecutionFailed(format!(
                "File '{}' does not exist or is not a file",
                path_str
            )));
        }

        let content = fs::read_to_string(&target_file)
            .map_err(|e| ToolError::ToolExecutionFailed(format!("Failed reading file: {}", e)))?;

        Ok(serde_json::json!({
            "path": path_str,
            "content": content
        }))
    }

    fn execute_workspace_write(
        args: &Value,
        ctx: &ToolExecutionContext,
    ) -> Result<Value, ToolError> {
        if !ctx.has_capability(&ToolCapability::FilesystemWrite)
            && !ctx.has_capability(&ToolCapability::WorkspaceWrite)
        {
            return Err(ToolError::CapabilityDenied {
                capability: "FilesystemWrite".to_string(),
                reason: "Missing write capability".to_string(),
            });
        }

        let path_str = args
            .get("path")
            .and_then(|p| p.as_str())
            .ok_or_else(|| ToolError::InvalidArguments("Missing 'path' argument".to_string()))?;

        let content = args
            .get("content")
            .and_then(|c| c.as_str())
            .ok_or_else(|| ToolError::InvalidArguments("Missing 'content' argument".to_string()))?;

        let target_file = Self::resolve_workspace_path(path_str, ctx)?;

        if let Some(parent) = target_file.parent() {
            let _ = fs::create_dir_all(parent);
        }

        fs::write(&target_file, content)
            .map_err(|e| ToolError::ToolExecutionFailed(format!("Failed writing file: {}", e)))?;

        Ok(serde_json::json!({
            "path": path_str,
            "bytes_written": content.len(),
            "status": "SUCCESS"
        }))
    }

    fn execute_workspace_search(
        args: &Value,
        ctx: &ToolExecutionContext,
    ) -> Result<Value, ToolError> {
        if !ctx.has_capability(&ToolCapability::FilesystemRead)
            && !ctx.has_capability(&ToolCapability::WorkspaceRead)
        {
            return Err(ToolError::CapabilityDenied {
                capability: "FilesystemRead".to_string(),
                reason: "Missing read capability for search".to_string(),
            });
        }

        let query = args
            .get("query")
            .and_then(|q| q.as_str())
            .ok_or_else(|| ToolError::InvalidArguments("Missing 'query' argument".to_string()))?;

        let ws_root = ctx.workspace_path.as_ref().ok_or_else(|| {
            ToolError::FilesystemDenied("No active workspace path set".to_string())
        })?;

        let mut matches = Vec::new();
        if ws_root.exists() {
            Self::search_dir_recursive(ws_root, ws_root, query, &mut matches, 50);
        }

        Ok(serde_json::json!({
            "query": query,
            "match_count": matches.len(),
            "matches": matches
        }))
    }

    fn search_dir_recursive(
        root: &Path,
        dir: &Path,
        query: &str,
        matches: &mut Vec<Value>,
        max_matches: usize,
    ) {
        if matches.len() >= max_matches {
            return;
        }

        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                let name = path.file_name().unwrap_or_default().to_string_lossy();
                if name.starts_with('.') || name == "node_modules" || name == "target" {
                    continue;
                }

                if path.is_dir() {
                    Self::search_dir_recursive(root, &path, query, matches, max_matches);
                } else if path.is_file() {
                    if let Ok(content) = fs::read_to_string(&path) {
                        for (idx, line) in content.lines().enumerate() {
                            if line.contains(query) {
                                let rel = path
                                    .strip_prefix(root)
                                    .unwrap_or(&path)
                                    .to_string_lossy()
                                    .to_string();
                                matches.push(serde_json::json!({
                                    "file": rel,
                                    "line_number": idx + 1,
                                    "line_text": line.trim()
                                }));
                                if matches.len() >= max_matches {
                                    return;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    fn execute_system_info(_args: &Value, ctx: &ToolExecutionContext) -> Result<Value, ToolError> {
        if !ctx.has_capability(&ToolCapability::SystemInfo) {
            return Err(ToolError::CapabilityDenied {
                capability: "SystemInfo".to_string(),
                reason: "Missing SystemInfo capability".to_string(),
            });
        }

        Ok(serde_json::json!({
            "os": std::env::consts::OS,
            "arch": std::env::consts::ARCH,
            "app_name": "OCTREX CODE V4",
            "runtime": "Local sovereign agentic workbench"
        }))
    }
}
