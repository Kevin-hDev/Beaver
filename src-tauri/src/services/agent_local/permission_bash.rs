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
        r"^npm\s+run\b",
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

pub fn is_read_only(command: &str) -> bool {
    let trimmed = command.trim();
    if crate::services::agent_local::sensitive_data::bash_touches_sensitive_data(trimmed)
        || has_control_operator(trimmed)
        || trimmed.contains("--output")
        || trimmed.contains("-delete")
        || trimmed.contains("-exec")
        || trimmed.contains("-ok")
        || trimmed.contains("-fprint")
        || trimmed.contains("-fprintf")
        || trimmed.contains("-fls")
    {
        return false;
    }
    let mut words = trimmed.split_whitespace();
    match (words.next(), words.next()) {
        (Some("git"), Some("status" | "log" | "diff" | "show")) => true,
        (Some("git"), Some("branch")) => bounded_arguments(words, is_read_only_branch_argument),
        (Some("git"), Some("remote")) => read_only_git_remote(&mut words),
        (Some("git"), Some("tag")) => read_only_git_tag(&mut words),
        (Some(command), _) => matches!(
            command,
            "ls" | "cat" | "head" | "tail" | "wc" | "grep" | "find" | "pwd"
                | "echo" | "which" | "tree" | "file" | "stat" | "du" | "df"
        ),
        _ => false,
    }
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

fn is_read_only_branch_argument(argument: &str) -> bool {
    matches!(
        argument,
        "--list" | "-l" | "--show-current" | "--all" | "-a" | "--remotes" | "-r"
            | "--verbose" | "-v" | "-vv" | "--no-color" | "--color"
    )
}

fn bounded_arguments<'a>(
    arguments: impl Iterator<Item = &'a str>,
    validate: impl Fn(&str) -> bool,
) -> bool {
    let mut count = 0;
    for argument in arguments {
        count += 1;
        if count > 32 || argument.len() > 256 || !validate(argument) {
            return false;
        }
    }
    true
}

fn read_only_git_remote(arguments: &mut std::str::SplitWhitespace<'_>) -> bool {
    match arguments.next() {
        None => true,
        Some("-v" | "--verbose") => arguments.next().is_none(),
        Some("show") => optional_single_argument(arguments),
        Some("get-url") => {
            let Some(argument) = arguments.next() else {
                return false;
            };
            let remote = if argument == "--all" {
                arguments.next()
            } else {
                Some(argument)
            };
            remote.is_some_and(valid_name) && arguments.next().is_none()
        }
        _ => false,
    }
}

fn read_only_git_tag(arguments: &mut std::str::SplitWhitespace<'_>) -> bool {
    match arguments.next() {
        None => true,
        Some("--list" | "-l") => bounded_arguments(arguments, valid_name),
        _ => false,
    }
}

fn optional_single_argument(arguments: &mut std::str::SplitWhitespace<'_>) -> bool {
    arguments.next().is_none_or(valid_name) && arguments.next().is_none()
}

fn valid_name(argument: &str) -> bool {
    argument.len() <= 256 && !argument.contains('\0')
}
