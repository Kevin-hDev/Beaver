use serde_json::Value;

/// Content and file-name search tools — always enabled (locked).
pub fn search_tool_definitions() -> Vec<Value> {
    use super::tool_definitions::tool_def;
    vec![
        tool_def(
            "grep",
            "Search file contents with a regex (Rust regex / RE2 syntax — same flavor as ripgrep). No backreferences, no lookahead. \
             Always case-sensitive. No -i, -A, -B, -C, -n options — line numbers are always included in output. \
             Output: one match per line, format `<path>:<line>:<content>`. There is no files_with_matches or count mode. \
             Use `glob` to restrict by filename (e.g. '*.rs', '*.{ts,tsx}'). Patterns map to the ignore crate glob. \
             Note: .gitignore is NOT respected — hidden and gitignored files are included in the search. \
             Max 250 matches (truncated with a notice). Pattern max 500 chars. Searches the working directory by default.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "pattern": {"type": "string", "description": "Regex pattern to search for"},
                    "path": {"type": "string", "description": "Directory to search (default: working directory)"},
                    "glob": {"type": "string", "description": "File filter glob (e.g. '*.rs', '*.ts')"}
                },
                "required": ["pattern"]
            }),
        ),
        tool_def(
            "glob",
            "Find files by name pattern. Patterns use globset syntax: `*`, `?`, `**`, `[abc]`, `{a,b}`. Matches return files only (not directories). \
             Output: absolute paths, one per line. Order is filesystem-dependent (not sorted). \
             Note: .gitignore is NOT respected — hidden and gitignored files are included. Use bash `ls` if you need a gitignore-aware view. \
             Max 100 results (truncated with a notice). Searches the working directory by default.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "pattern": {"type": "string", "description": "Glob pattern (e.g. '**/*.ts', 'src/**/*.rs')"},
                    "path": {"type": "string", "description": "Root directory (default: working directory)"}
                },
                "required": ["pattern"]
            }),
        ),
    ]
}
