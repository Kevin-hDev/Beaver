use std::time::Duration;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::oneshot;
use tokio_util::sync::CancellationToken;
use zeroize::Zeroizing;

use super::OAuthFailure;

const BIND_ADDR: &str = "127.0.0.1:56121";
const TIMEOUT: Duration = Duration::from_secs(300);
const MAX_REQUEST_BYTES: usize = 8 * 1024;
const MAX_CODE_BYTES: usize = 4 * 1024;
const MAX_QUERY_PAIRS: usize = 8;

pub async fn start(
    expected_state: Zeroizing<String>,
    cancel: CancellationToken,
) -> Result<oneshot::Receiver<Result<Zeroizing<String>, OAuthFailure>>, OAuthFailure> {
    verify_state(&expected_state, &expected_state)?;
    let listener = TcpListener::bind(BIND_ADDR)
        .await
        .map_err(|_| OAuthFailure::Generic)?;
    let (sender, receiver) = oneshot::channel();
    tokio::spawn(async move {
        let result = tokio::time::timeout(TIMEOUT, async {
            tokio::select! {
                value = accept_until_valid(&listener, &expected_state) => value,
                _ = cancel.cancelled() => Err(OAuthFailure::Cancelled),
            }
        })
        .await
        .unwrap_or(Err(OAuthFailure::Expired));
        let _ = sender.send(result);
    });
    Ok(receiver)
}

async fn accept_until_valid(
    listener: &TcpListener,
    expected_state: &str,
) -> Result<Zeroizing<String>, OAuthFailure> {
    loop {
        let (mut stream, _) = listener.accept().await.map_err(|_| OAuthFailure::Generic)?;
        match handle_connection(&mut stream, expected_state).await {
            Ok(code) => return Ok(code),
            Err(OAuthFailure::Denied) => return Err(OAuthFailure::Denied),
            Err(_) => {}
        }
    }
}

async fn handle_connection(
    stream: &mut TcpStream,
    expected_state: &str,
) -> Result<Zeroizing<String>, OAuthFailure> {
    let result = read_request(stream)
        .await
        .and_then(|request| parse_request(&request, expected_state));
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

async fn read_request(stream: &mut TcpStream) -> Result<Zeroizing<Vec<u8>>, OAuthFailure> {
    let mut request = Zeroizing::new(Vec::with_capacity(1_024));
    let mut chunk = [0_u8; 1_024];
    loop {
        let read = stream
            .read(&mut chunk)
            .await
            .map_err(|_| OAuthFailure::Generic)?;
        if read == 0 || request.len().saturating_add(read) > MAX_REQUEST_BYTES {
            return Err(OAuthFailure::Generic);
        }
        request.extend_from_slice(&chunk[..read]);
        if request.windows(4).any(|window| window == b"\r\n\r\n") {
            return Ok(request);
        }
    }
}

fn parse_request(request: &[u8], expected_state: &str) -> Result<Zeroizing<String>, OAuthFailure> {
    let request = std::str::from_utf8(request).map_err(|_| OAuthFailure::Generic)?;
    let mut parts = request
        .lines()
        .next()
        .ok_or(OAuthFailure::Generic)?
        .split_whitespace();
    let method = parts.next();
    let target = parts.next();
    let version = parts.next();
    if method != Some("GET") || version != Some("HTTP/1.1") || parts.next().is_some() {
        return Err(OAuthFailure::Generic);
    }
    parse_target(target.ok_or(OAuthFailure::Generic)?, expected_state)
}

fn parse_target(target: &str, expected_state: &str) -> Result<Zeroizing<String>, OAuthFailure> {
    let (path, query) = target.split_once('?').ok_or(OAuthFailure::Generic)?;
    if path != "/callback" {
        return Err(OAuthFailure::Generic);
    }
    let mut code = None;
    let mut state = None;
    let mut denied = false;
    for (index, (key, value)) in url::form_urlencoded::parse(query.as_bytes()).enumerate() {
        if index >= MAX_QUERY_PAIRS {
            return Err(OAuthFailure::Generic);
        }
        match key.as_ref() {
            "code" if code.is_none() => code = Some(value.into_owned()),
            "state" if state.is_none() => state = Some(value.into_owned()),
            "error" => denied = true,
            "code" | "state" => return Err(OAuthFailure::Generic),
            _ => {}
        }
    }
    verify_state(state.as_deref().unwrap_or_default(), expected_state)?;
    if denied {
        return Err(OAuthFailure::Denied);
    }
    let code = code
        .filter(|value| !value.is_empty() && value.len() <= MAX_CODE_BYTES)
        .ok_or(OAuthFailure::Generic)?;
    Ok(Zeroizing::new(code))
}

pub fn verify_state(actual: &str, expected: &str) -> Result<(), OAuthFailure> {
    crate::services::mcp_oauth::flow_auth::verify_state_constant_time(expected, actual)
        .map_err(|_| OAuthFailure::Generic)
}

#[cfg(test)]
#[path = "callback_tests.rs"]
mod tests;
