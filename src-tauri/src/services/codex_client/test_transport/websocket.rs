use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use tokio::sync::oneshot;
use tokio_tungstenite::tungstenite::http::header::AUTHORIZATION;
use tokio_tungstenite::tungstenite::Message as WsMessage;
use zeroize::Zeroizing;

use super::{
    projection, record_websocket, state, ScenarioContext, WebSocketCapture, WebSocketReply,
};
use crate::services::codex_client::websocket_connect::{CodexSocket, ConnectError};
use crate::services::codex_oauth::store::CodexTokens;
use crate::services::codex_oauth::token::constant_time_secret_eq;

struct HandshakeCapture {
    routing_hint: Option<String>,
    authorization_valid: bool,
    account_header_present: bool,
    originator_valid: bool,
    user_agent_present: bool,
    beta_header_valid: bool,
    session_headers_valid: bool,
}

pub(super) async fn connect_websocket(
    context: std::sync::Arc<ScenarioContext>,
    session_id: &str,
    routing_hint: &str,
) -> Result<CodexSocket, ConnectError> {
    let reply = {
        let mut state = state(&context);
        match state.websocket_script.take() {
            Some(reply) => reply,
            None => return Err(ConnectError::Unavailable),
        }
    };
    let listener = match tokio::net::TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0)).await {
        Ok(listener) => listener,
        Err(_) => return Err(ConnectError::Unavailable),
    };
    let address = match listener.local_addr() {
        Ok(address) => address,
        Err(_) => return Err(ConnectError::Unavailable),
    };
    let expected_session = session_id.to_string();
    tokio::spawn(async move {
        let _ = serve(listener, expected_session, reply, context).await;
    });
    let url = format!("ws://{address}");
    crate::services::codex_client::websocket_connect::connect_loopback_at(
        &url,
        &credentials(),
        Some(session_id),
        routing_hint,
    )
    .await
}

#[allow(clippy::result_large_err)] // Signature imposée par le callback de tungstenite.
async fn serve(
    listener: tokio::net::TcpListener,
    expected_session: String,
    reply: WebSocketReply,
    context: std::sync::Arc<ScenarioContext>,
) -> Result<(), String> {
    let (stream, _) = tokio::time::timeout(Duration::from_secs(2), listener.accept())
        .await
        .map_err(|_| invalid())?
        .map_err(|_| invalid())?;
    let (header_tx, header_rx) = oneshot::channel();
    let mut header_tx = Some(header_tx);
    let mut socket = tokio_tungstenite::accept_hdr_async(
        stream,
        move |request: &tokio_tungstenite::tungstenite::handshake::server::Request, response| {
            let headers = request.headers();
            let routing_hint = headers
                .get("x-codex-routing-hint")
                .and_then(|value| value.to_str().ok())
                .filter(|value| value.len() <= 160)
                .map(str::to_string);
            let authorization_valid = headers.get(AUTHORIZATION).is_some_and(|value| {
                constant_time_secret_eq(value.as_bytes(), b"Bearer access-test")
            });
            let session_headers_valid = ["session-id", "thread-id", "x-client-request-id"]
                .into_iter()
                .all(|name| {
                    headers.get(name).and_then(|value| value.to_str().ok())
                        == Some(expected_session.as_str())
                });
            let capture = HandshakeCapture {
                routing_hint,
                authorization_valid,
                account_header_present: headers.contains_key("chatgpt-account-id"),
                originator_valid: headers
                    .get("originator")
                    .and_then(|value| value.to_str().ok())
                    == Some(crate::services::codex_oauth::ORIGINATOR),
                user_agent_present: headers.contains_key("user-agent"),
                beta_header_valid: headers
                    .get("openai-beta")
                    .and_then(|value| value.to_str().ok())
                    == Some("responses_websockets=2026-02-06"),
                session_headers_valid,
            };
            if let Some(sender) = header_tx.take() {
                let _ = sender.send(capture);
            }
            Ok(response)
        },
    )
    .await
    .map_err(|_| invalid())?;
    let message = tokio::time::timeout(Duration::from_secs(2), socket.next())
        .await
        .map_err(|_| invalid())?
        .ok_or_else(invalid)?
        .map_err(|_| invalid())?;
    let WsMessage::Text(text) = message else {
        return Err(invalid());
    };
    if text.len() > crate::services::secure_http::LLM_BODY_LIMIT {
        return Err(invalid());
    }
    let request = projection::parse(text.as_bytes())?;
    drop(text);
    let handshake = header_rx.await.map_err(|_| invalid())?;
    record_websocket(
        &context,
        WebSocketCapture {
            request,
            routing_hint: handshake.routing_hint,
            authorization_valid: handshake.authorization_valid,
            account_header_present: handshake.account_header_present,
            originator_valid: handshake.originator_valid,
            user_agent_present: handshake.user_agent_present,
            beta_header_valid: handshake.beta_header_valid,
            session_headers_valid: handshake.session_headers_valid,
        },
    )?;

    match reply {
        WebSocketReply::Success => socket
            .send(WsMessage::Text(
                "{\"type\":\"response.completed\",\"response\":{\"usage\":{}}}"
                    .to_string()
                    .into(),
            ))
            .await
            .map_err(|_| invalid()),
        WebSocketReply::Unavailable => socket.close(None).await.map_err(|_| invalid()),
    }
}

fn credentials() -> CodexTokens {
    CodexTokens {
        access: Zeroizing::new("access-test".to_string()),
        refresh: Zeroizing::new("refresh-test".to_string()),
        expires_at: i64::MAX,
        refresh_not_before: 0,
        account_hint: Zeroizing::new("acct_test".to_string()),
    }
}

fn invalid() -> String {
    "provider_configuration_invalid".to_string()
}
