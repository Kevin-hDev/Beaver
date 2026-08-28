use std::time::Duration;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use zeroize::Zeroizing;

use super::*;
use crate::services::codex_oauth::store::CodexTokens;
use crate::services::llm::fast_mode::FastModeRequest;
use crate::services::secure_http::AuthenticatedClient;

const MAX_TEST_REQUEST_BYTES: usize = 16 * 1024;

fn credentials() -> CodexTokens {
    CodexTokens {
        access: Zeroizing::new("access-test".to_string()),
        refresh: Zeroizing::new("refresh-test".to_string()),
        expires_at: i64::MAX,
        refresh_not_before: 0,
        account_hint: Zeroizing::new("acct_test".to_string()),
        credential_scope: Some(
            crate::services::api_keys::generate_credential_scope().expect("scope"),
        ),
    }
}

#[test]
fn codex_transport_errors_use_stable_codes() {
    assert_eq!(
        secure_http_error(SecureHttpError::Configuration),
        "provider_configuration_invalid"
    );
    assert_eq!(
        secure_http_error(SecureHttpError::Status),
        "provider_request_rejected"
    );
    assert_eq!(
        secure_http_error(SecureHttpError::Request),
        "provider_connection_failed"
    );
}

#[tokio::test]
async fn http_post_carries_the_canonical_fast_body_and_routing_hint() {
    let fast = capture_request(FastModeRequest::Fast).await;
    let standard = capture_request(FastModeRequest::Standard).await;

    let fast_body: serde_json::Value = serde_json::from_str(request_body(&fast)).unwrap();
    assert_eq!(fast_body["service_tier"], "priority");
    assert_eq!(
        header_value(&fast, "x-codex-routing-hint"),
        Some("model=gpt-5.6-sol;tier=priority")
    );

    let standard_body: serde_json::Value = serde_json::from_str(request_body(&standard)).unwrap();
    assert!(standard_body.get("service_tier").is_none());
    assert_eq!(
        header_value(&standard, "x-codex-routing-hint"),
        Some("model=gpt-5.6-sol")
    );
    assert!(!header_value(&standard, "x-codex-routing-hint")
        .unwrap()
        .contains(";tier="));
}

async fn capture_request(fast_mode: FastModeRequest) -> String {
    let listener = tokio::net::TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0))
        .await
        .unwrap();
    let address = listener.local_addr().unwrap();
    let server = tokio::spawn(async move {
        let (socket, _) = listener.accept().await.unwrap();
        read_http_request(socket).await
    });
    let request = crate::services::codex_client::request::build_codex_request(
        "gpt-5.6-sol",
        &[],
        &[],
        None,
        None,
        fast_mode,
    );
    let body = serde_json::to_string(&request).unwrap();
    let routing_hint = crate::services::codex_client::routing_hint::for_request(&request).unwrap();
    let client = AuthenticatedClient::new_loopback(Duration::from_secs(2)).unwrap();
    let url = format!("http://{address}/responses");

    let response = send_once(&client, &credentials(), &url, &body, &routing_hint)
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let received = server.await.unwrap();
    assert!(received.starts_with("POST /responses HTTP/1.1\r\n"));
    assert!(header_value(&received, "chatgpt-account-id").is_some());
    assert!(header_value(&received, "originator").is_some());
    assert!(!request_body(&received).contains("access_token"));
    let production_url = reqwest::Url::parse(CODEX_API_BASE).unwrap();
    assert_eq!(
        production_url.origin().ascii_serialization(),
        "https://chatgpt.com"
    );
    received
}

async fn read_http_request(mut socket: tokio::net::TcpStream) -> String {
    let mut bytes = Vec::with_capacity(2 * 1024);
    loop {
        assert!(bytes.len() < MAX_TEST_REQUEST_BYTES);
        let mut chunk = [0_u8; 1024];
        let read = socket.read(&mut chunk).await.unwrap();
        assert_ne!(read, 0);
        bytes.extend_from_slice(&chunk[..read]);
        if request_is_complete(&bytes) {
            break;
        }
    }
    socket
        .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n")
        .await
        .unwrap();
    String::from_utf8(bytes).unwrap()
}

fn request_is_complete(bytes: &[u8]) -> bool {
    let Some(header_end) = bytes.windows(4).position(|window| window == b"\r\n\r\n") else {
        return false;
    };
    let headers = std::str::from_utf8(&bytes[..header_end]).unwrap();
    let content_length = header_value(headers, "content-length")
        .unwrap()
        .parse::<usize>()
        .unwrap();
    bytes.len() >= header_end + 4 + content_length
}

fn header_value<'a>(request: &'a str, name: &str) -> Option<&'a str> {
    request.lines().find_map(|line| {
        let (header_name, value) = line.split_once(':')?;
        header_name
            .eq_ignore_ascii_case(name)
            .then_some(value.trim())
    })
}

fn request_body(request: &str) -> &str {
    request.split_once("\r\n\r\n").unwrap().1
}
