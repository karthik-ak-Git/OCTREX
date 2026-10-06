# OCTREX CODE V4 — Tool System Specification

## 1. Tool Engine Contracts

The Tool Engine exposes controlled APIs for filesystem, terminal, git, testing, and repository inspection. All tool executions return structured JSON results and propagate error events cleanly.

---

## 2. Core Tool Definitions

### 2.1 Filesystem Tools
- `file_read`: Reads file contents with line range filtering (`StartLine`, `EndLine`).
- `file_write`: Writes or appends full file contents.
- `file_patch`: Performs precise block line replacements (`TargetContent`, `ReplacementContent`, `StartLine`, `EndLine`).
- `file_create_dir`: Ensures directory structure exists.
- `file_delete`: Removes target files/directories subject to risk controls.
- `file_search`: Locates files matching glob/regex patterns.

### 2.2 Terminal Tools
- `terminal_execute`: Runs shell commands within target working directories. Supports real-time stdout/stderr streaming.
- `terminal_cancel`: Aborts active execution by process ID.
- `terminal_status`: Returns current status and buffer log location.

### 2.3 Git Tools
- `git_status`: Returns tracked/untracked state and branch info.
- `git_diff`: Generates detailed unified diffs against baseline.
- `git_checkpoint`: Creates temporary Git stashes or commits before major edits.
- `git_rollback`: Reverts working directory to previous safe checkpoint.
- `git_worktree_create`: Creates isolated workspace for multi-agent or tournament workflows.
- `git_worktree_remove`: Safely tears down temporary worktrees.

### 2.4 Testing Tools
- `test_run`: Executes project test runners (npm test, pytest, go test, cargo test, etc.).
- `test_target`: Executes specific test suites or individual test files.
- `build_run`: Executes project build tools.
- `lint_run`: Runs linters and typecheckers.

### 2.5 Repository Tools
- `repo_symbol_search`: Locates class, function, or struct definitions via AST/ripgrep.
- `repo_find_references`: Identifies all caller sites for a symbol.
- `repo_inspect_deps`: Parses package manifests (`package.json`, `Cargo.toml`, `requirements.txt`, etc.).

---

## 3. Safe Edit Protocols & Patch Strategy

1. **Read-Before-Write:** Agents must inspect target line ranges before generating edits.
2. **Atomic Block Replacement:** Whole-file overwrites are restricted; agents use precise line patch replacements to prevent overwriting parallel user edits.
3. **Pre-Modification Checkpoints:** Git checkpoints are automatically created prior to modifying high-risk files or executing multi-file refactors.
