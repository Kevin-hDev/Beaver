use regex::Regex;
use std::sync::LazyLock;

static SAFE_PATTERNS: LazyLock<Vec<Regex>> = LazyLock::new(|| {
    [
        r"^ls\b",
        r"^cat\b",
        r"^head\b",
        r"^tail\b",
        r"^wc\b",
        r"^grep\b",
        r"^find\b",
        r"^git\s+(status|log|diff|show|remote|tag)\b",
        r"^git\s+branch\s*$",
        r"^pwd$",
        r"^echo\b",
        r"^which\b",
        r"^cargo\s+(check|test|clippy|build)\b",
        r"^npx\s+tsc\b",
        r"^npm\s+(run|test)\b",
        r"^tree\b",
        r"^file\b",
        r"^stat\b",
        r"^du\b",
        r"^df\b",
    ]
    .into_iter()
    .filter_map(|pattern| Regex::new(pattern).ok())
    .collect()
});

pub fn is_safe(command: &str) -> bool {
    let trimmed = command.trim();
    if crate::services::agent_local::sensitive_data::bash_touches_sensitive_data(trimmed)
        || has_control_operator(trimmed)
    {
        return false;
    }
    SAFE_PATTERNS.iter().any(|pattern| pattern.is_match(trimmed))
}

fn has_control_operator(command: &str) -> bool {
    command.contains(';')
        || command.contains("&&")
        || command.contains("||")
        || command.contains('|')
        || command.contains('`')
        || command.contains("$(")
        || command.contains('\n')
        || command.contains('\r')
        || command.contains("<(")
        || command.contains(">(")
        || command.contains("<<")
        || command.contains('>')
        || command.contains("$'")
        || command.contains('&')
        || command.contains('<')
}
