pub(super) fn bounded_websocket_config(
    max_bytes: usize,
) -> tokio_tungstenite::tungstenite::protocol::WebSocketConfig {
    tokio_tungstenite::tungstenite::protocol::WebSocketConfig::default()
        .max_message_size(Some(max_bytes))
        .max_frame_size(Some(max_bytes))
}

#[cfg(test)]
mod tests {
    use super::bounded_websocket_config;

    #[test]
    fn websocket_messages_and_frames_are_bounded() {
        let config = bounded_websocket_config(1024);
        assert_eq!(config.max_message_size, Some(1024));
        assert_eq!(config.max_frame_size, Some(1024));
    }
}
