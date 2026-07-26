use crate::services::agent_local::types_ollama::ChatMessage;

pub fn estimate_tokens(messages: &[ChatMessage]) -> usize {
    crate::services::token_counting::estimate_chat_tokens(messages)
}

pub fn should_compress(used_tokens: usize, context_window: u64, threshold_pct: u8) -> bool {
    if threshold_pct == 0 {
        return false;
    }
    let limit = (context_window as f64 * threshold_pct as f64 / 100.0) as usize;
    used_tokens >= limit
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::agent_local::types_ollama::ChatMessage;

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
    fn estimate_empty() {
        assert_eq!(estimate_tokens(&[]), 0);
    }

    #[test]
    fn estimate_simple_message() {
        let msgs = vec![msg("user", &"a".repeat(400))];
        assert_eq!(estimate_tokens(&msgs), 100);
    }

    #[test]
    fn estimate_multiple_messages() {
        let msgs = vec![
            msg("user", &"a".repeat(400)),
            msg("assistant", &"b".repeat(800)),
            msg("tool", &"c".repeat(1200)),
        ];
        assert_eq!(estimate_tokens(&msgs), 600);
    }

    #[test]
    fn threshold_check() {
        assert!(!should_compress(80_000, 100_000, 85));
        assert!(should_compress(86_000, 100_000, 85));
    }

    #[test]
    fn threshold_zero_means_never() {
        assert!(!should_compress(99_000, 100_000, 0));
    }

    #[test]
    fn threshold_100_means_at_max() {
        assert!(!should_compress(99_999, 100_000, 100));
        assert!(should_compress(100_000, 100_000, 100));
    }

    #[test]
    fn estimate_counts_images() {
        let mut message = msg("user", "hello");
        message.images = Some(vec!["iVBORw0KGgo=".to_string()]);
        assert!(estimate_tokens(&[message]) >= crate::services::llm::vision::IMAGE_TOKEN_ESTIMATE);
    }

    #[test]
    fn estimate_counts_cjk_conservatively() {
        let msgs = vec![msg("user", &"你".repeat(1000))];
        assert_eq!(estimate_tokens(&msgs), 1250);
    }
}
