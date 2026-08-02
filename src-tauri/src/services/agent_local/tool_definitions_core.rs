use serde_json::Value;

/// Base filesystem and shell tools — always enabled (locked).
pub fn core_tool_definitions() -> Vec<Value> {
    use super::tool_definitions::tool_def;
    vec![
        tool_def(
            "bash",
            "Execute a shell command on the user's machine.\n\n\
             Role and shell: use bash for system commands and shell operations. The complete user environment is loaded from a cached profile. Unix uses a POSIX-compatible $SHELL when available, otherwise zsh, bash, or sh; Windows uses PowerShell.\n\n\
             Working directory: commands start in the project directory. Set workdir to an absolute directory only when this call intentionally needs another location. A cd never persists to later calls.\n\n\
             Permissions: in Ask for approval mode, read-only commands run directly while mutating commands require explicit approval. In Full access mode, commands run without approval prompts.\n\n\
             Safety: system-level destructive commands such as chmod 777, mkfs, dd, and fork bombs are blocked.\n\n\
             Output and sessions: output streams live. Short commands return immediately. If a command remains active after yield_time_ms (default 10000), the result contains a session_id; continue it with bash_control. There is no forced execution timeout unless timeout is explicitly set. Full output remains available outside the model context when its preview is truncated.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "command": {"type": "string", "description": "Shell command to execute"},
                    "timeout": {"type": "integer", "description": "Optional hard timeout in seconds; omitted means no forced timeout"},
                    "yield_time_ms": {"type": "integer", "description": "Wait before returning a running session (250-30000 ms, default 10000)"},
                    "workdir": {"type": "string", "description": "Optional absolute working directory for this call only"}
                },
                "required": ["command"]
            }),
        ),
        tool_def(
            "bash_control",
            "Continue or control a shell process returned by bash. Poll with only session_id, send input with chars, set eof=true to close its input after any chars, or set stop=true to terminate the process and all of its children. Commands started with & remain managed by the session until their background jobs finish or the session is stopped. Output streams live while waiting. A completed session returns its final exit status and is then removed.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "session_id": {"type": "string", "description": "Session identifier returned by bash"},
                    "chars": {"type": "string", "description": "Optional input to write to the process"},
                    "eof": {"type": "boolean", "description": "Close process input after writing optional chars"},
                    "stop": {"type": "boolean", "description": "Stop the process and all of its children"},
                    "yield_time_ms": {"type": "integer", "description": "Wait for output or completion (250-30000 ms, default 10000)"}
                },
                "required": ["session_id"]
            }),
        ),
        tool_def(
            "read_file",
            "Read a UTF-8 text file from disk. Returns content with line numbers (1-based, tab-separated). \
             Limit: 20 MB max. Files larger than this return an error. \
             Binary/non-UTF-8 files (images, PDFs, .docx, executables) cannot be read — use a dedicated document, image, or spreadsheet extension tool when available. \
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
