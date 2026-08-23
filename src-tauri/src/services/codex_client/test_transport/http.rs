use std::time::Duration;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use zeroize::Zeroizing;

use super::{record_http, record_refresh, state, HttpCapture, HttpReply};
use crate::services::codex_oauth::store::CodexTokens;
use crate::services::codex_oauth::token::constant_time_secret_eq;
use crate::services::secure_http::AuthenticatedClient;

const MAX_TEST_REQUEST_BYTES: usize = 64 * 1024;
const SUCCESS_BODY: &str = concat!(
    "data: {\"type\":\"response.completed\",",
    "\"response\":{\"usage\":{\"input_tokens\":1,\"output_tokens\":1}}}\n\n",
);

pub(super) async fn dispatch_http(
    body: &str,
    routing_hint: &str,
    model: &str,
    tool_count: usize,
) -> Option<Result<reqwest::Response, String>> {
    let script = {
        let mut state = state();
        if !state.active {
            return None;
        }
        match state.http_script.take() {
            Some(script) => script,
            None => return Some(Err("provider_configuration_invalid".to_string())),
        }
    };
    let listener = tokio::net::TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0))
        .await
        .map_err(|_| "provider_configuration_invalid".to_string());
    let listener = match listener {
        Ok(listener) => listener,
        Err(error) => return Some(Err(error)),
    };
    let address = match listener.local_addr() {
        Ok(address) => address,
        Err(_) => return Some(Err("provider_configuration_invalid".to_string())),
    };
    let server = tokio::spawn(serve(listener, script));
    let client = match AuthenticatedClient::new_loopback(Duration::from_secs(2)) {
        Ok(client) => client,
        Err(_) => return Some(Err("provider_configuration_invalid".to_string())),
    };
    let initial = credentials("access-test");
    let endpoint = format!("http://{address}/responses");
    let refresh = async {
        record_refresh();
        Ok(credentials("access-refreshed"))
    };
    let response = crate::services::codex_client::request_http::post_with_refresh(
        &client,
        &initial,
        &endpoint,
        body,
        routing_hint,
        model,
        tool_count,
        refresh,
    )
    .await;
    let server_result = tokio::time::timeout(Duration::from_secs(2), server).await;
    if !matches!(server_result, Ok(Ok(Ok(())))) {
        return Some(Err("provider_configuration_invalid".to_string()));
    }
    Some(response)
}

async fn serve(listener: tokio::net::TcpListener, script: Vec<HttpReply>) -> Result<(), String> {
    for (index, reply) in script.into_iter().enumerate() {
        let (socket, _) = tokio::time::timeout(Duration::from_secs(2), listener.accept())
            .await
            .map_err(|_| "provider_configuration_invalid".to_string())?
            .map_err(|_| "provider_configuration_invalid".to_string())?;
        let expected_access = if index == 0 {
            b"Bearer access-test".as_slice()
        } else {
            b"Bearer access-refreshed".as_slice()
        };
        serve_once(socket, reply, expected_access).await?;
    }
    Ok(())
}

async fn serve_once(
    mut socket: tokio::net::TcpStream,
    reply: HttpReply,
    expected_access: &[u8],
) -> Result<(), String> {
    let bytes = read_request(&mut socket).await?;
    let split = bytes
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .ok_or_else(invalid)?;
    let headers = std::str::from_utf8(&bytes[..split]).map_err(|_| invalid())?;
    let body_bytes = &bytes[split + 4..];
    let body: serde_json::Value = serde_json::from_slice(body_bytes).map_err(|_| invalid())?;
    let routing_hint = header_value(headers, "x-codex-routing-hint").map(str::to_string);
    if routing_hint.as_ref().is_some_and(|value| value.len() > 160) {
        return Err(invalid());
    }
    let authorization_valid = header_value(headers, "authorization")
        .is_some_and(|value| constant_time_secret_eq(value.as_bytes(), expected_access));
    let path = headers
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .ok_or_else(invalid)?
        .to_string();
    record_http(HttpCapture {
        body_has_access_token: body.to_string().contains("\"access_token\""),
        body,
        routing_hint,
        authorization_valid,
        account_header_present: header_value(headers, "chatgpt-account-id").is_some(),
        originator_valid: header_value(headers, "originator")
            == Some(crate::services::codex_oauth::ORIGINATOR),
        user_agent_present: header_value(headers, "user-agent").is_some(),
        path,
        body_bytes: body_bytes.len(),
    })?;

    let (status, content_type, response_body) = match reply {
        HttpReply::Unauthorized => ("401 Unauthorized", "application/json", "{}"),
        HttpReply::Success => ("200 OK", "text/event-stream", SUCCESS_BODY),
    };
    let response = format!(
        "HTTP/1.1 {status}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\n\r\n{response_body}",
        response_body.len()
    );
    socket
        .write_all(response.as_bytes())
        .await
        .map_err(|_| invalid())
}

async fn read_request(socket: &mut tokio::net::TcpStream) -> Result<Zeroizing<Vec<u8>>, String> {
    let mut bytes = Zeroizing::new(Vec::with_capacity(2 * 1024));
    loop {
        if bytes.len() >= MAX_TEST_REQUEST_BYTES {
            return Err(invalid());
        }
        let mut chunk = [0_u8; 1024];
        let read = socket.read(&mut chunk).await.map_err(|_| invalid())?;
        if read == 0 {
            return Err(invalid());
        }
        bytes.extend_from_slice(&chunk[..read]);
        if request_is_complete(&bytes)? {
            return Ok(bytes);
        }
    }
}

fn request_is_complete(bytes: &[u8]) -> Result<bool, String> {
    let Some(header_end) = bytes.windows(4).position(|window| window == b"\r\n\r\n") else {
        return Ok(false);
    };
    let headers = std::str::from_utf8(&bytes[..header_end]).map_err(|_| invalid())?;
    let content_length = header_value(headers, "content-length")
        .ok_or_else(invalid)?
        .parse::<usize>()
        .map_err(|_| invalid())?;
    if content_length > MAX_TEST_REQUEST_BYTES {
        return Err(invalid());
    }
    Ok(bytes.len() >= header_end + 4 + content_length)
}

fn header_value<'a>(headers: &'a str, name: &str) -> Option<&'a str> {
    headers.lines().find_map(|line| {
        let (header_name, value) = line.split_once(':')?;
        header_name
            .eq_ignore_ascii_case(name)
            .then_some(value.trim())
    })
}

fn credentials(access: &str) -> CodexTokens {
    CodexTokens {
        access: Zeroizing::new(access.to_string()),
        refresh: Zeroizing::new("refresh-test".to_string()),
        expires_at: i64::MAX,
        refresh_not_before: 0,
        account_hint: Zeroizing::new("acct_test".to_string()),
    }
}

fn invalid() -> String {
    "provider_configuration_invalid".to_string()
}
