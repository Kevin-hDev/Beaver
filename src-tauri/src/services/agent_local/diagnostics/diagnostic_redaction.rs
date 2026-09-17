use serde_json::Value;

const REDACTED: &str = "[redacted]";
const PATH_REDACTED: &str = "[path]";

pub(crate) fn redact_value(value: Value) -> Value {
    match value {
        Value::String(text) => Value::String(redact_text(&text)),
        Value::Array(items) => Value::Array(items.into_iter().map(redact_value).collect()),
        Value::Object(map) => Value::Object(
            map.into_iter()
                .map(|(key, value)| {
                    let safe_value = if is_sensitive_key(&key) {
                        Value::String(REDACTED.to_string())
                    } else {
                        redact_value(value)
                    };
                    (key, safe_value)
                })
                .collect(),
        ),
        other => other,
    }
}

pub(crate) fn redact_text(value: &str) -> String {
    value
        .split_whitespace()
        .map(redact_token)
        .collect::<Vec<_>>()
        .join(" ")
}

fn redact_token(token: &str) -> String {
    let trimmed = token.trim_matches(|c: char| matches!(c, '"' | '\'' | ',' | ';' | ')' | '('));
    if looks_like_path(trimmed) {
        return token.replace(trimmed, PATH_REDACTED);
    }
    if looks_like_secret(trimmed) {
        return token.replace(trimmed, REDACTED);
    }
    token.to_string()
}

fn looks_like_path(value: &str) -> bool {
    value.starts_with("/Users/")
        || value.starts_with("/home/")
        || value.starts_with("/var/")
        || value.starts_with("/tmp/")
        || value.starts_with("/private/")
        || value
            .as_bytes()
            .get(1)
            .is_some_and(|b| *b == b':' && value.as_bytes()[0].is_ascii_alphabetic())
}

fn looks_like_secret(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    if lower.starts_with("sk-") || lower.starts_with("xox") {
        return true;
    }
    if let Some((key, val)) = lower.split_once('=') {
        return is_sensitive_key(key) && val.len() > 2;
    }
    [
        "bearer", "token", "secret", "password", "api_key", "apikey", ".env",
    ]
    .iter()
    .any(|needle| lower.contains(needle))
}

fn is_sensitive_key(key: &str) -> bool {
    let lower = key.to_ascii_lowercase();
    [
        "authorization",
        "api_key",
        "apikey",
        "password",
        "secret",
        "token",
        "content",
        "old_text",
        "new_text",
        "old_string",
        "new_string",
        "body",
    ]
    .iter()
    .any(|needle| lower.contains(needle))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn redacts_paths_and_secret_tokens_in_text() {
        let out = redact_text("failed at /Users/kevinh/app/file.rs with sk-secret123");
        assert!(!out.contains("/Users/kevinh"));
        assert!(!out.contains("sk-secret123"));
        assert!(out.contains(PATH_REDACTED));
        assert!(out.contains(REDACTED));
    }

    #[test]
    fn redacts_sensitive_json_keys_recursively() {
        let out = redact_value(json!({
            "path": "/Users/kevinh/private.txt",
            "nested": {"content": "secret file content"},
            "command": "echo token=abc123"
        }));

        assert_eq!(out["path"], PATH_REDACTED);
        assert_eq!(out["nested"]["content"], REDACTED);
        assert_eq!(out["command"], "echo [redacted]");
    }
}
