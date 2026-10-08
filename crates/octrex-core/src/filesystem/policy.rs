use crate::filesystem::errors::FilesystemError;
use crate::filesystem::types::{
    FilesystemDisposition, FilesystemOperation, ProtectedPath, ProtectedPathAction,
    WorkspaceSecurityPolicy,
};
use regex::Regex;

pub struct PolicyEvaluator;

impl PolicyEvaluator {
    /// Evaluates if a target relative path matches any protected path rules.
    pub fn evaluate_protected_path(
        rel_path: &str,
        policy: &WorkspaceSecurityPolicy,
    ) -> Option<(ProtectedPath, ProtectedPathAction)> {
        let clean_path = rel_path.trim().replace('\\', "/");
        let file_name = clean_path.split('/').last().unwrap_or(&clean_path);

        for rule in &policy.protected_paths {
            if Self::path_matches_pattern(&clean_path, file_name, &rule.pattern) {
                return Some((rule.clone(), rule.action));
            }
        }

        None
    }

    /// Helper to match path/filename against wildcard glob patterns like `.env`, `*.pem`, `id_rsa*`, `.git/*`.
    fn path_matches_pattern(full_path: &str, file_name: &str, pattern: &str) -> bool {
        let pat = pattern.trim().replace('\\', "/");

        if pat == full_path || pat == file_name {
            return true;
        }

        // Convert wildcard pattern to regex
        let mut regex_str = String::from("^");
        for ch in pat.chars() {
            match ch {
                '*' => regex_str.push_str(".*"),
                '?' => regex_str.push('.'),
                '.' | '+' | '(' | ')' | '|' | '^' | '$' | '{' | '}' | '[' | ']' | '\\' => {
                    regex_str.push('\\');
                    regex_str.push(ch);
                }
                _ => regex_str.push(ch),
            }
        }
        regex_str.push('$');

        if let Ok(re) = Regex::new(&regex_str) {
            if re.is_match(full_path) || re.is_match(file_name) {
                return true;
            }
        }

        false
    }

    /// Evaluates overall filesystem disposition for an operation based on workspace policy and protected path rules.
    pub fn evaluate_policy_disposition(
        rel_path: &str,
        operation: FilesystemOperation,
        policy: &WorkspaceSecurityPolicy,
    ) -> Result<FilesystemDisposition, FilesystemError> {
        // 1. Read-only workspace check for write-like operations
        if policy.read_only && operation.is_write_like() {
            return Ok(FilesystemDisposition::Block);
        }

        // 2. Specific operation permissions
        match operation {
            FilesystemOperation::Delete | FilesystemOperation::DeleteDirectory => {
                if !policy.allow_delete {
                    return Ok(FilesystemDisposition::RequireConfirmation);
                }
            }
            FilesystemOperation::RecursiveDelete => {
                if !policy.allow_recursive_delete {
                    return Ok(FilesystemDisposition::RequireConfirmation);
                }
            }
            FilesystemOperation::Export => {
                if !policy.allow_export {
                    return Ok(FilesystemDisposition::Block);
                }
            }
            FilesystemOperation::Import => {
                if !policy.allow_import {
                    return Ok(FilesystemDisposition::Block);
                }
            }
            _ => {}
        }

        // 3. Protected paths check
        if let Some((_rule, action)) = Self::evaluate_protected_path(rel_path, policy) {
            match action {
                ProtectedPathAction::Block => return Ok(FilesystemDisposition::Block),
                ProtectedPathAction::ReadOnly => {
                    if operation.is_write_like() {
                        return Ok(FilesystemDisposition::Block);
                    }
                }
                ProtectedPathAction::RequireConfirmation => {
                    return Ok(FilesystemDisposition::RequireConfirmation);
                }
            }
        }

        Ok(FilesystemDisposition::Allow)
    }
}
