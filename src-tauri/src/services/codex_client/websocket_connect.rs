use std::time::Duration;

use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::http::header::{AUTHORIZATION, USER_AGENT};
use tokio_tungstenite::tungstenite::http::{HeaderName, HeaderValue, StatusCode};
use zeroize::Zeroizing;

use crate::services::codex_oauth::store::CodexTokens;
use crate::services::codex_oauth::token;
use crate::services::secure_http::LLM_BODY_LIMIT;

const CODEX_WEBSOCKET_URL: &str = "wss://chatgpt.com/backend-api/codex/responses";
const WEBSOCKET_CONNECT_TIMEOUT: Duration = Duration::from_secs(5);
const WEBSOCKET_BETA: &str = "responses_websockets=2026-02-06";
const MAX_METADATA_HEADER_BYTES: usize = 256;

pub(super) type CodexSocket =
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>;

pub(super) async fn connect(session_id: &str) -> Result<CodexSocket, ConnectError> {
    let credentials = token::ensure_valid()
        .await
        .map_err(|_| ConnectError::Unavailable)?;
    match connect_once_at(CODEX_WEBSOCKET_URL, &credentials, Some(session_id)).await {
        Ok(socket) => Ok(socket),
        Err(ConnectError::Unauthorized) => {
            let refreshed = token::recover_after_unauthorized(credentials.access.as_str())
                .await
                .map_err(|_| ConnectError::Unavailable)?;
            connect_once_at(CODEX_WEBSOCKET_URL, &refreshed, Some(session_id)).await
        }
        Err(error) => Err(error),
    }
}

pub(super) async fn connect_once_at(
    url: &str,
    credentials: &CodexTokens,
    session_id: Option<&str>,
) -> Result<CodexSocket, ConnectError> {
    let mut request = url
        .into_client_request()
        .map_err(|_| ConnectError::Unavailable)?;
    let headers = request.headers_mut();
    insert_sensitive(
        headers,
        AUTHORIZATION,
        &Zeroizing::new(format!("Bearer {}", credentials.access.as_str())),
    )?;
    insert_sensitive(
        headers,
        HeaderName::from_static("chatgpt-account-id"),
        credentials.account_hint.as_str(),
    )?;
    insert(
        headers,
        HeaderName::from_static("openai-beta"),
        WEBSOCKET_BETA,
    )?;
    insert(
        headers,
        HeaderName::from_static("originator"),
        crate::services::codex_oauth::ORIGINATOR,
    )?;
    insert(headers, USER_AGENT, &crate::services::brand::user_agent())?;
    if let Some(session_id) = session_id {
        insert_metadata(headers, "session-id", session_id)?;
        insert_metadata(headers, "thread-id", session_id)?;
        insert_metadata(headers, "x-client-request-id", session_id)?;
    }

    let connection = tokio::time::timeout(
        WEBSOCKET_CONNECT_TIMEOUT,
        tokio_tungstenite::connect_async_with_config(request, Some(websocket_config()), false),
    )
    .await
    .map_err(|_| ConnectError::Unavailable)?;
    connection
        .map(|(socket, _)| socket)
        .map_err(map_connect_error)
}

fn websocket_config() -> tokio_tungstenite::tungstenite::protocol::WebSocketConfig {
    tokio_tungstenite::tungstenite::protocol::WebSocketConfig::default()
        .max_message_size(Some(LLM_BODY_LIMIT))
        .max_frame_size(Some(LLM_BODY_LIMIT))
}

fn insert(
    headers: &mut tokio_tungstenite::tungstenite::http::HeaderMap,
    name: HeaderName,
    value: &str,
) -> Result<(), ConnectError> {
    let value = HeaderValue::from_str(value).map_err(|_| ConnectError::Unavailable)?;
    headers.insert(name, value);
    Ok(())
}

fn insert_sensitive(
    headers: &mut tokio_tungstenite::tungstenite::http::HeaderMap,
    name: HeaderName,
    value: &str,
) -> Result<(), ConnectError> {
    let mut value = HeaderValue::from_str(value).map_err(|_| ConnectError::Unavailable)?;
    value.set_sensitive(true);
    headers.insert(name, value);
    Ok(())
}

fn insert_metadata(
    headers: &mut tokio_tungstenite::tungstenite::http::HeaderMap,
    name: &'static str,
    value: &str,
) -> Result<(), ConnectError> {
    if value.is_empty() || value.len() > MAX_METADATA_HEADER_BYTES {
        return Err(ConnectError::Unavailable);
    }
    insert(headers, HeaderName::from_static(name), value)
}

fn map_connect_error(error: tokio_tungstenite::tungstenite::Error) -> ConnectError {
    if matches!(
        error,
        tokio_tungstenite::tungstenite::Error::Http(ref response)
            if response.status() == StatusCode::UNAUTHORIZED
    ) {
        ConnectError::Unauthorized
    } else {
        ConnectError::Unavailable
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ConnectError {
    Unauthorized,
    Unavailable,
}

#[cfg(test)]
#[path = "websocket_connect_tests.rs"]
mod tests;
