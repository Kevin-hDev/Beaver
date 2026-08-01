use futures_util::StreamExt;
use tokio::sync::oneshot;
use tokio_tungstenite::tungstenite::http::header::AUTHORIZATION;
use zeroize::Zeroizing;

use super::*;
use crate::services::codex_oauth::store::CodexTokens;
use crate::services::codex_oauth::token::constant_time_secret_eq;

fn credentials() -> CodexTokens {
    CodexTokens {
        access: Zeroizing::new("access-test".to_string()),
        refresh: Zeroizing::new("refresh-test".to_string()),
        expires_at: i64::MAX,
        refresh_not_before: 0,
        account_hint: Zeroizing::new("acct_test".to_string()),
    }
}

#[test]
fn websocket_limits_match_the_bounded_llm_transport() {
    let config = websocket_config();
    assert_eq!(config.max_message_size, Some(LLM_BODY_LIMIT));
    assert_eq!(config.max_frame_size, Some(LLM_BODY_LIMIT));
}

#[tokio::test]
#[allow(clippy::result_large_err)] // Signature imposée par le callback de tungstenite.
async fn handshake_carries_current_codex_headers_without_logging_secrets() {
    let listener = tokio::net::TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0))
        .await
        .unwrap();
    let address = listener.local_addr().unwrap();
    let (header_tx, header_rx) = oneshot::channel();
    let server = tokio::spawn(async move {
        let (stream, _) = listener.accept().await.unwrap();
        let mut header_tx = Some(header_tx);
        let mut socket = tokio_tungstenite::accept_hdr_async(
            stream,
            move |request: &tokio_tungstenite::tungstenite::handshake::server::Request,
                  response| {
                let authorization = request
                    .headers()
                    .get(AUTHORIZATION)
                    .map(|value| constant_time_secret_eq(value.as_bytes(), b"Bearer access-test"))
                    .unwrap_or(false);
                let beta = request
                    .headers()
                    .get("openai-beta")
                    .and_then(|v| v.to_str().ok())
                    == Some(WEBSOCKET_BETA);
                let originator = request
                    .headers()
                    .get("originator")
                    .and_then(|v| v.to_str().ok())
                    == Some(crate::services::codex_oauth::ORIGINATOR);
                let session = request
                    .headers()
                    .get("session-id")
                    .and_then(|v| v.to_str().ok())
                    == Some("session-test");
                if let Some(sender) = header_tx.take() {
                    let _ = sender.send((authorization, beta, originator, session));
                }
                Ok(response)
            },
        )
        .await
        .unwrap();
        let _ = socket.next().await;
    });

    let url = format!("ws://{address}");
    let mut socket = connect_loopback_at(&url, &credentials(), Some("session-test"))
        .await
        .unwrap();
    let headers = header_rx.await.unwrap();
    assert_eq!(headers, (true, true, true, true));
    socket.close(None).await.unwrap();
    server.await.unwrap();
}

#[test]
fn production_websocket_policy_rejects_every_other_target() {
    assert!(websocket_url_allowed(
        CODEX_WEBSOCKET_URL,
        WebSocketUrlPolicy::CodexProduction
    ));
    assert!(!websocket_url_allowed(
        "ws://chatgpt.com/backend-api/codex/responses",
        WebSocketUrlPolicy::CodexProduction
    ));
    assert!(!websocket_url_allowed(
        "wss://example.com/backend-api/codex/responses",
        WebSocketUrlPolicy::CodexProduction
    ));
}

#[test]
fn test_websocket_policy_only_accepts_plain_loopback() {
    assert!(websocket_url_allowed(
        "ws://127.0.0.1:1455/test",
        WebSocketUrlPolicy::LoopbackTest
    ));
    assert!(websocket_url_allowed(
        "ws://[::1]:1455/test",
        WebSocketUrlPolicy::LoopbackTest
    ));
    assert!(!websocket_url_allowed(
        "ws://example.com/test",
        WebSocketUrlPolicy::LoopbackTest
    ));
    assert!(!websocket_url_allowed(
        "ws://127.0.0.1:1455/test?token=secret",
        WebSocketUrlPolicy::LoopbackTest
    ));
}
