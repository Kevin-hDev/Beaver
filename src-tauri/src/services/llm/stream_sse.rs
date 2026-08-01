const MAX_EVENT_BYTES: usize = crate::services::secure_http::LLM_BODY_LIMIT;

pub(crate) fn is_done_marker(data: &str) -> bool {
    data.trim() == "[DONE]"
}

pub(crate) fn parse_json(data: &str) -> Result<serde_json::Value, String> {
    if data.len() > MAX_EVENT_BYTES {
        return Err("provider_connection_failed".to_string());
    }
    serde_json::from_str(data).map_err(|_| "provider_connection_failed".to_string())
}

#[cfg(test)]
mod tests {
    use super::{is_done_marker, parse_json, MAX_EVENT_BYTES};

    #[test]
    fn recognizes_done_marker_with_whitespace() {
        assert!(is_done_marker(" [DONE]\n"));
    }

    #[test]
    fn rejects_regular_json_chunk() {
        assert!(!is_done_marker(r#"{"choices":[]}"#));
    }

    #[test]
    fn rejects_an_oversized_external_event_before_json_allocation() {
        assert!(parse_json(&" ".repeat(MAX_EVENT_BYTES + 1)).is_err());
    }
}
