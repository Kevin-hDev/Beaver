use serde_json::Value;

/// Base filesystem and shell tools — always enabled (locked).
pub fn core_tool_definitions() -> Vec<Value> {
    use super::tool_definitions::tool_def;
    vec![
        tool_def(
            "bash",
            "Execute a shell command on the user's machine. \
             Shell: $SHELL -c (Unix) or PowerShell (Windows). Commands run in the working directory. \
             IMPORTANT — prefer dedicated tools, they give the user a better experience and are easier to approve: \
             find files with glob (not find); search contents with grep (not grep/rg); \
             read a file with read_file (not cat/head/tail); edit a file with edit_file (not sed/awk); \
             create a file with write_file (not echo/cat <<EOF). \
             Permissions: in Ask for approval mode, read-only commands (ls, cat, git status/log/diff, pwd, etc.) run without confirmation, \
             while mutating commands require explicit user approval. In Full access mode, bash commands run without approval prompts. \
             Safety: system-level destructive commands are blocked (chmod 777, mkfs, dd, fork bombs, etc.). \
             Never skip git hooks (--no-verify), never force-push to main/master, never amend an existing commit unless the user explicitly asks. \
             Investigate hook failures instead of bypassing them. \
             Never run interactive editors (vim, nano, less) — the shell is non-interactive. \
             Long-running commands (npm run dev, vite, cargo tauri dev, flask run, tail -f, --watch, etc.) are auto-detected and run in the background. \
             The tool returns once a 'ready' marker is seen (localhost:, listening, compiled successfully) or after up to 30s. The process keeps running; do not block on it. \
             Timeout: default 120s, max 600s. For long builds/tests pass an explicit timeout. \
             Output is truncated to 2000 lines / 50KB. \
             Note: the working directory does NOT persist across bash calls — each call starts from the session working directory. \
             Use absolute paths or `cd X && cmd` in a single call.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "command": {"type": "string", "description": "Shell command to execute"},
                    "timeout": {"type": "integer", "description": "Timeout in seconds (default: 120, max: 600)"}
                },
                "required": ["command"]
            }),
        ),
        tool_def(
            "read_file",
            "Read a UTF-8 text file from disk. Returns content with line numbers (1-based, tab-separated). \
             Limit: 20 MB max. Files larger than this return an error. \
             Binary/non-UTF-8 files (images, PDFs, .docx, executables) cannot be read — use the dedicated Office tools (read_document, read_image, read_spreadsheet) when available. \
             Non-existent files return a generic error. \
             Use offset/limit to page through large files. Default limit 2000 lines; max 50000 lines. \
             Output format: each line is prefixed with `<line_number>\\t<content>`. If more lines remain, a hint with the next offset is appended. \
             Read paths must be inside the working directory or an explicitly allowed read root (data dir, temp, advanced.allowed_paths).",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {"type": "string", "description": "File path (relative to working directory, or absolute)"},
                    "offset": {"type": "integer", "description": "Starting line (0-based, default: 0)"},
                    "limit": {"type": "integer", "description": "Max lines to return (default: 2000, max: 50000)"}
                },
                "required": ["path"]
            }),
        ),
        tool_def(
            "write_file",
            "Create or overwrite a file. Relative paths resolve from the working directory. \
             Read-before-write rule: if the target file already exists, you MUST have called read_file on it earlier in this session. The call fails otherwise. New files can be written without a prior read. \
             Writes are restricted to allowed write roots (working directory and configured paths under advanced.allowed_paths). Writing outside (e.g. ~/.bashrc, ~/.ssh) is refused. \
             Symlinks are not followed on write. \
             Prefer edit_file for modifying an existing file — it only sends the diff and keeps edits surgical. Use write_file only to create new files or do complete rewrites. \
             Requires user confirmation unless session-allowed.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {"type": "string", "description": "File path (relative to working directory, or absolute)"},
                    "content": {"type": "string", "description": "Content to write"}
                },
                "required": ["path", "content"]
            }),
        ),
        tool_def(
            "edit_file",
            "Modify a file by replacing one exact occurrence of a string. Relative paths resolve from the working directory. \
             Requirements: \
             - You MUST have called read_file on this path earlier in the session (read-before-edit). \
             - `old_string` must be unique in the file. If multiple matches are found, the call fails with the match count — include more surrounding context (usually 2-4 adjacent lines) to make it unique. \
             - The match is exact: whitespace, tabs, and newlines must match the file content byte-for-byte. \
             Does not support replace_all. To rename a symbol across a file, call edit_file once per occurrence, or use write_file with the full new content. \
             Returns the edited line number. \
             Requires user confirmation unless session-allowed.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {"type": "string", "description": "File path (relative to working directory, or absolute)"},
                    "old_string": {"type": "string", "description": "Exact text to find (must be unique in file)"},
                    "new_string": {"type": "string", "description": "Replacement text (must differ from old_string)"}
                },
                "required": ["path", "old_string", "new_string"]
            }),
        ),
        tool_def(
            "list_dir",
            "List directory contents as a small recursive tree. \
             Output: indented entries, directories suffixed with `/`. Sorted alphabetically. No file sizes or metadata. \
             Depth: recursive up to 3 levels deep. Flat listing is not available — for a flat view use bash `ls`. \
             Excluded by default: dotfiles (names starting with `.`), `node_modules`, `target`. \
             Truncated at 500 entries. \
             Read paths must be inside the working directory or an allowed read root. Use '.' to list the working directory.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {"type": "string", "description": "Directory path (use '.' for working directory)"}
                },
                "required": ["path"]
            }),
        ),
    ]
}
