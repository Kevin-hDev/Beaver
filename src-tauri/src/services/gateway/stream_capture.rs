use std::sync::LazyLock;

use regex::Regex;

use crate::services::agent_local::types_ollama::ChatMessage;
use crate::services::gateway::security::validation::split_utf16;

static PATH_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?:/[a-zA-Z0-9_./-]{10,}|[A-Z]:\\[a-zA-Z0-9_.\\ /-]{10,})").expect("path regex")
});

static IP_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\b(?:10\.\d{1,3}\.\d{1,3}\.\d{1,3}|172\.(?:1[6-9]|2\d|3[01])\.\d{1,3}\.\d{1,3}|192\.168\.\d{1,3}\.\d{1,3}|127\.\d{1,3}\.\d{1,3}\.\d{1,3})\b").expect("ip regex")
});

pub fn extract_final_reply(messages: &[ChatMessage]) -> Option<String> {
    messages
        .iter()
        .rev()
        .find(|m| m.role == "assistant" && !m.content.is_empty())
        .map(|m| m.content.clone())
}

pub fn redact_sensitive(content: &str) -> String {
    let step1 = crate::services::agent_local::sensitive_data::redact_text(content);
    let step2 = PATH_RE.replace_all(&step1, "[REDACTED]");
    IP_RE.replace_all(&step2, "[REDACTED]").into_owned()
}

pub fn prepare_for_channel(content: &str, max_utf16: usize) -> Vec<String> {
    let redacted = redact_sensitive(content);
    split_utf16(&redacted, max_utf16)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn msg(role: &str, content: &str) -> ChatMessage {
        ChatMessage {
            role: role.to_string(),
            content: content.to_string(),
            images: None,
            tool_calls: None,
            tool_name: None,
            tool_call_id: None,
            reasoning_content: None,
        }
    }

    #[test]
    fn extract_last_assistant() {
        let msgs = vec![
            msg("user", "hello"),
            msg("assistant", "hi"),
            msg("tool", "result"),
        ];
        assert_eq!(extract_final_reply(&msgs), Some("hi".to_string()));
    }

    #[test]
    fn extract_empty_returns_none() {
        let msgs: Vec<ChatMessage> = vec![];
        assert_eq!(extract_final_reply(&msgs), None);
    }

    #[test]
    fn redact_api_key() {
        let text = "token is sk-proj-abc123def456";
        assert!(redact_sensitive(text).contains("[REDACTED]"));
        assert!(!redact_sensitive(text).contains("sk-proj"));
    }

    #[test]
    fn redact_absolute_path() {
        let text = "file at /home/user/project/secret.rs";
        assert!(redact_sensitive(text).contains("[REDACTED]"));
    }

    #[test]
    fn redact_private_ip() {
        let text = "connect to 192.168.1.42";
        assert!(redact_sensitive(text).contains("[REDACTED]"));
    }

    #[test]
    fn prepare_splits_long_text() {
        let text = "a".repeat(5000);
        let chunks = prepare_for_channel(&text, 4096);
        assert!(chunks.len() >= 2);
    }
}
