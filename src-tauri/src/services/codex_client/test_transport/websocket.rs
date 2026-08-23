use std::sync::Arc;
use std::time::Duration;

use zeroize::Zeroizing;

use super::{
    projection, record_websocket, state, websocket_raw, ScenarioContext, WebSocketCapture,
    WebSocketReply,
};
use crate::services::codex_client::websocket_connect::{CodexSocket, ConnectError};
use crate::services::codex_oauth::store::CodexTokens;

pub(super) async fn connect_websocket(
    context: Arc<ScenarioContext>,
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
        let _ = serve(listener, &expected_session, reply, context).await;
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

async fn serve(
    listener: tokio::net::TcpListener,
    expected_session: &str,
    reply: WebSocketReply,
    context: Arc<ScenarioContext>,
) -> Result<(), String> {
    let (stream, _) = tokio::time::timeout(Duration::from_secs(2), listener.accept())
        .await
        .map_err(|_| invalid())?
        .map_err(|_| invalid())?;
    let (mut stream, handshake) = tokio::time::timeout(
        Duration::from_secs(2),
        websocket_raw::accept(stream, expected_session),
    )
    .await
    .map_err(|_| invalid())??;
    let payload = tokio::time::timeout(
        Duration::from_secs(2),
        websocket_raw::read_text(&mut stream, Arc::clone(&context.websocket_payload_zeroized)),
    )
    .await
    .map_err(|_| invalid())??;
    let request = projection::parse(&payload)?;
    drop(payload);
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
    websocket_raw::write_reply(&mut stream, reply).await
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
