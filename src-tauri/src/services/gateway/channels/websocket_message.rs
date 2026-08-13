use tokio_tungstenite::tungstenite::{Error as WsError, Message as WsMessage, Utf8Bytes};

#[derive(Debug, Eq, PartialEq)]
#[must_use = "the channel must distinguish control frames from disconnection"]
pub(super) enum IncomingWebSocket {
    Text(Utf8Bytes),
    Ignore,
    Disconnect,
}

pub(super) fn classify_incoming(message: Option<Result<WsMessage, WsError>>) -> IncomingWebSocket {
    match message {
        Some(Ok(WsMessage::Text(text))) => IncomingWebSocket::Text(text),
        Some(Ok(WsMessage::Close(_))) | Some(Err(_)) | None => IncomingWebSocket::Disconnect,
        Some(Ok(_)) => IncomingWebSocket::Ignore,
    }
}
