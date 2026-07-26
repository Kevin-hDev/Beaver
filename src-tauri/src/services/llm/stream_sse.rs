pub(crate) fn is_done_marker(data: &str) -> bool {
    data.trim() == "[DONE]"
}

#[cfg(test)]
mod tests {
    use super::is_done_marker;

    #[test]
    fn recognizes_done_marker_with_whitespace() {
        assert!(is_done_marker(" [DONE]\n"));
    }

    #[test]
    fn rejects_regular_json_chunk() {
        assert!(!is_done_marker(r#"{"choices":[]}"#));
    }
}
