use crate::filesystem::errors::FilesystemError;
use std::path::Path;

/// Distinguishes Application-Level Filesystem Mediation from OS-Level Process Sandboxing.
///
/// Application-level mediation is enforced through `FilesystemSecurityService` for all tool, API, and agent operations.
/// OS-level process sandboxing (such as landlock, seccomp, Windows AppContainer) is an optional lower-level layer.
pub struct ProcessSandboxBoundary;

impl ProcessSandboxBoundary {
    /// Inspects a raw shell command for obvious filesystem escapes outside the workspace root.
    pub fn inspect_command_filesystem_boundary(
        command: &str,
        workspace_root: &Path,
    ) -> Result<(), FilesystemError> {
        let cmd = command.trim();
        let root_str = workspace_root.to_string_lossy();

        // Detect explicit system file targets in command strings
        let suspicious_targets = [
            "/etc/passwd",
            "/etc/shadow",
            "c:\\windows\\system32",
            "c:/windows/system32",
            "~/.ssh",
            "id_rsa",
        ];

        for target in suspicious_targets {
            if cmd.to_lowercase().contains(target) && !root_str.to_lowercase().contains(target) {
                return Err(FilesystemError::OperationBlocked {
                    operation: "terminal_execute".to_string(),
                    reason: format!(
                        "Terminal command contains target outside workspace boundary: '{}'",
                        target
                    ),
                });
            }
        }

        Ok(())
    }
}
