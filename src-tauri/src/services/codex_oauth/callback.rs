use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio_util::sync::CancellationToken;
use zeroize::Zeroizing;

const BIND_ADDR: &str = "127.0.0.1:1455";
const TIMEOUT: Duration = Duration::from_secs(120);
const MAX_REQUEST_BYTES: usize = 8 * 1024;
const MAX_CODE_BYTES: usize = 4 * 1024;
const STATE_BYTES: usize = 32;
const MAX_QUERY_PAIRS: usize = 8;

#[derive(Debug)]
pub struct CallbackResult {
    pub code: Zeroizing<String>,
}

pub async fn wait_for_callback(
    expected_state: &str,
    cancel: &CancellationToken,
) -> Result<CallbackResult, String> {
    validate_state(expected_state)?;
    let listener = TcpListener::bind(BIND_ADDR)
        .await
        .map_err(|_| "callback OAuth indisponible".to_string())?;
    tokio::time::timeout(TIMEOUT, async {
        tokio::select! {
            result = accept_until_valid(&listener, expected_state) => result,
            _ = cancel.cancelled() => Err("callback OAuth annulé".to_string()),
        }
    })
    .await
    .map_err(|_| "callback OAuth expiré".to_string())?
}

async fn accept_until_valid(
    listener: &TcpListener,
    expected_state: &str,
) -> Result<CallbackResult, String> {
    loop {
        let (mut stream, _) = listener
            .accept()
            .await
            .map_err(|_| "callback OAuth indisponible".to_string())?;
        match handle_connection(&mut stream, expected_state).await {
            Ok(result) => return Ok(result),
            Err(()) => continue,
        }
    }
}

async fn handle_connection(
    stream: &mut TcpStream,
    expected_state: &str,
) -> Result<CallbackResult, ()> {
    let result = read_request(stream)
        .await
        .and_then(|request| parse_callback_bytes(&request, expected_state).map_err(|_| ()));
    let (status, body) = if result.is_ok() {
        (
            "200 OK",
            "Connexion réussie. Vous pouvez fermer cet onglet.",
        )
    } else {
        ("400 Bad Request", "Requête OAuth invalide.")
    };
    let response = format!(
        "HTTP/1.1 {status}\r\nContent-Type: text/plain; charset=utf-8\r\nConnection: close\r\nContent-Length: {}\r\n\r\n{body}",
        body.len()
    );
    let _ = stream.write_all(response.as_bytes()).await;
    let _ = stream.shutdown().await;
    result
}

async fn read_request(stream: &mut TcpStream) -> Result<Zeroizing<Vec<u8>>, ()> {
    let mut request = Zeroizing::new(Vec::with_capacity(1024));
    let mut chunk = [0_u8; 1024];
    loop {
        let read = stream.read(&mut chunk).await.map_err(|_| ())?;
        if read == 0 || request.len().saturating_add(read) > MAX_REQUEST_BYTES {
            return Err(());
        }
        request.extend_from_slice(&chunk[..read]);
        if request.windows(4).any(|window| window == b"\r\n\r\n") {
            return Ok(request);
        }
    }
}

fn parse_callback_bytes(request: &[u8], expected_state: &str) -> Result<CallbackResult, String> {
    let request =
        std::str::from_utf8(request).map_err(|_| "callback OAuth invalide".to_string())?;
    let first_line = request
        .split("\r\n")
        .next()
        .ok_or_else(|| "callback OAuth invalide".to_string())?;
    let mut parts = first_line.split_whitespace();
    if parts.next() != Some("GET") {
        return Err("callback OAuth invalide".to_string());
    }
    let target = parts
        .next()
        .ok_or_else(|| "callback OAuth invalide".to_string())?;
    if parts.next() != Some("HTTP/1.1") || parts.next().is_some() {
        return Err("callback OAuth invalide".to_string());
    }
    parse_target(target, expected_state)
}

fn parse_target(target: &str, expected_state: &str) -> Result<CallbackResult, String> {
    let (path, query) = target
        .split_once('?')
        .ok_or_else(|| "callback OAuth invalide".to_string())?;
    if path != "/auth/callback" {
        return Err("callback OAuth invalide".to_string());
    }
    let mut code = None;
    let mut state = None;
    let mut count = 0_usize;
    for pair in query.split('&') {
        count += 1;
        if count > MAX_QUERY_PAIRS {
            return Err("callback OAuth invalide".to_string());
        }
        let (key, value) = pair
            .split_once('=')
            .ok_or_else(|| "callback OAuth invalide".to_string())?;
        match key {
            "code" if code.is_none() => code = Some(value),
            "state" if state.is_none() => state = Some(value),
            "error" => return Err("callback OAuth refusé".to_string()),
            "code" | "state" => return Err("callback OAuth invalide".to_string()),
            _ => {}
        }
    }
    let state = state.ok_or_else(|| "callback OAuth invalide".to_string())?;
    if !constant_time_state_eq(state, expected_state) {
        return Err("callback OAuth invalide".to_string());
    }
    let encoded = code.ok_or_else(|| "callback OAuth invalide".to_string())?;
    if encoded.is_empty() || encoded.len() > MAX_CODE_BYTES {
        return Err("callback OAuth invalide".to_string());
    }
    let decoded =
        urlencoding::decode(encoded).map_err(|_| "callback OAuth invalide".to_string())?;
    if decoded.is_empty() || decoded.len() > MAX_CODE_BYTES {
        return Err("callback OAuth invalide".to_string());
    }
    Ok(CallbackResult {
        code: Zeroizing::new(decoded.into_owned()),
    })
}

fn validate_state(state: &str) -> Result<(), String> {
    if state.len() == STATE_BYTES && state.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        Ok(())
    } else {
        Err("état OAuth invalide".to_string())
    }
}

fn constant_time_state_eq(actual: &str, expected: &str) -> bool {
    let mut actual_fixed = [0_u8; STATE_BYTES];
    let mut expected_fixed = [0_u8; STATE_BYTES];
    let actual_valid = copy_state(actual, &mut actual_fixed);
    let expected_valid = copy_state(expected, &mut expected_fixed);
    let mut diff = actual_valid | expected_valid;
    for index in 0..STATE_BYTES {
        diff |= actual_fixed[index] ^ expected_fixed[index];
    }
    diff == 0
}

fn copy_state(input: &str, output: &mut [u8; STATE_BYTES]) -> u8 {
    let bytes = input.as_bytes();
    let valid =
        u8::from(bytes.len() == STATE_BYTES && bytes.iter().all(|byte| byte.is_ascii_hexdigit()));
    for (index, byte) in bytes.iter().take(STATE_BYTES).enumerate() {
        output[index] = *byte;
    }
    valid ^ 1
}

#[cfg(test)]
#[path = "callback_tests.rs"]
mod tests;
