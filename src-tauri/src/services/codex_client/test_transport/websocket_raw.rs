use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use tokio::io::{AsyncReadExt, AsyncWriteExt};

use super::sensitive_buffer::SensitiveBuffer;
use super::WebSocketReply;
use crate::services::codex_oauth::token::constant_time_secret_eq;

const MAX_HANDSHAKE_BYTES: usize = 16 * 1024;

pub(super) struct HandshakeCapture {
    pub routing_hint: Option<String>,
    pub authorization_valid: bool,
    pub account_header_present: bool,
    pub originator_valid: bool,
    pub user_agent_present: bool,
    pub beta_header_valid: bool,
    pub session_headers_valid: bool,
}

pub(super) async fn accept(
    mut stream: tokio::net::TcpStream,
    expected_session: &str,
) -> Result<(tokio::net::TcpStream, HandshakeCapture), String> {
    let zeroized = Arc::new(AtomicBool::new(false));
    let mut request = SensitiveBuffer::with_capacity(2 * 1024, zeroized);
    let header_end = loop {
        if request.len() >= MAX_HANDSHAKE_BYTES {
            return Err(invalid());
        }
        let mut chunk = [0_u8; 1024];
        let read = stream.read(&mut chunk).await.map_err(|_| invalid())?;
        if read == 0 {
            return Err(invalid());
        }
        request.extend_from_slice(&chunk[..read]);
        if let Some(end) = request.windows(4).position(|window| window == b"\r\n\r\n") {
            break end;
        }
    };
    let headers = std::str::from_utf8(&request[..header_end]).map_err(|_| invalid())?;
    let key = header_value(headers, "sec-websocket-key").ok_or_else(invalid)?;
    let accept_key = tokio_tungstenite::tungstenite::handshake::derive_accept_key(key.as_bytes());
    let routing_hint = header_value(headers, "x-codex-routing-hint")
        .filter(|value| value.len() <= 160)
        .map(str::to_string);
    let authorization_valid = header_value(headers, "authorization")
        .is_some_and(|value| constant_time_secret_eq(value.as_bytes(), b"Bearer access-test"));
    let session_headers_valid = ["session-id", "thread-id", "x-client-request-id"]
        .into_iter()
        .all(|name| header_value(headers, name) == Some(expected_session));
    let capture = HandshakeCapture {
        routing_hint,
        authorization_valid,
        account_header_present: header_value(headers, "chatgpt-account-id").is_some(),
        originator_valid: header_value(headers, "originator")
            == Some(crate::services::codex_oauth::ORIGINATOR),
        user_agent_present: header_value(headers, "user-agent").is_some(),
        beta_header_valid: header_value(headers, "openai-beta")
            == Some("responses_websockets=2026-02-06"),
        session_headers_valid,
    };
    let response = format!(
        "HTTP/1.1 101 Switching Protocols\r\nConnection: Upgrade\r\nUpgrade: websocket\r\nSec-WebSocket-Accept: {accept_key}\r\n\r\n"
    );
    stream
        .write_all(response.as_bytes())
        .await
        .map_err(|_| invalid())?;
    drop(request);
    Ok((stream, capture))
}

pub(super) async fn read_text(
    stream: &mut tokio::net::TcpStream,
    zeroized: Arc<AtomicBool>,
) -> Result<SensitiveBuffer, String> {
    let mut header = [0_u8; 2];
    stream
        .read_exact(&mut header)
        .await
        .map_err(|_| invalid())?;
    let valid_text = header[0] & 0xf0 == 0x80 && header[0] & 0x0f == 1;
    if !valid_text || header[1] & 0x80 == 0 {
        return Err(invalid());
    }
    let payload_len = match header[1] & 0x7f {
        length @ 0..=125 => usize::from(length),
        126 => {
            let mut extended = [0_u8; 2];
            stream
                .read_exact(&mut extended)
                .await
                .map_err(|_| invalid())?;
            usize::from(u16::from_be_bytes(extended))
        }
        127 => {
            let mut extended = [0_u8; 8];
            stream
                .read_exact(&mut extended)
                .await
                .map_err(|_| invalid())?;
            usize::try_from(u64::from_be_bytes(extended)).map_err(|_| invalid())?
        }
        _ => return Err(invalid()),
    };
    if payload_len > crate::services::secure_http::LLM_BODY_LIMIT {
        return Err(invalid());
    }
    let mut mask = [0_u8; 4];
    stream.read_exact(&mut mask).await.map_err(|_| invalid())?;
    let mut payload = SensitiveBuffer::zeroed(payload_len, zeroized);
    stream
        .read_exact(&mut payload[..])
        .await
        .map_err(|_| invalid())?;
    for (index, byte) in payload.iter_mut().enumerate() {
        *byte ^= mask[index % mask.len()];
    }
    Ok(payload)
}

pub(super) async fn write_reply(
    stream: &mut tokio::net::TcpStream,
    reply: WebSocketReply,
) -> Result<(), String> {
    let (opcode, payload) = match reply {
        WebSocketReply::Success => (
            1_u8,
            b"{\"type\":\"response.completed\",\"response\":{\"usage\":{}}}".as_slice(),
        ),
        WebSocketReply::Unavailable => (8_u8, &[][..]),
    };
    if payload.len() > 125 {
        return Err(invalid());
    }
    let header = [0x80 | opcode, payload.len() as u8];
    stream.write_all(&header).await.map_err(|_| invalid())?;
    stream.write_all(payload).await.map_err(|_| invalid())
}

fn header_value<'a>(headers: &'a str, name: &str) -> Option<&'a str> {
    headers.lines().find_map(|line| {
        let (header_name, value) = line.split_once(':')?;
        header_name
            .eq_ignore_ascii_case(name)
            .then_some(value.trim())
    })
}

fn invalid() -> String {
    "provider_configuration_invalid".to_string()
}
