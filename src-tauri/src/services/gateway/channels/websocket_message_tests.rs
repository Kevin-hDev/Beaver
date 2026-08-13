use super::websocket_message::{classify_incoming, IncomingWebSocket};
use tokio_tungstenite::tungstenite::{Error as WsError, Message as WsMessage};

#[test]
fn websocket_control_frames_do_not_disconnect_healthy_channels() {
    assert!(matches!(
        classify_incoming(Some(Ok(WsMessage::Text("hello".into())))),
        IncomingWebSocket::Text(text) if text.as_str() == "hello"
    ));

    for message in [
        WsMessage::Ping(Vec::new().into()),
        WsMessage::Pong(Vec::new().into()),
        WsMessage::Binary(Vec::new().into()),
    ] {
        assert_eq!(
            classify_incoming(Some(Ok(message))),
            IncomingWebSocket::Ignore
        );
    }

    assert_eq!(
        classify_incoming(Some(Ok(WsMessage::Close(None)))),
        IncomingWebSocket::Disconnect
    );
    assert_eq!(
        classify_incoming(Some(Err(WsError::ConnectionClosed))),
        IncomingWebSocket::Disconnect
    );
    assert_eq!(classify_incoming(None), IncomingWebSocket::Disconnect);
}
