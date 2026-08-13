use std::time::Duration;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::task::JoinSet;
use tokio_util::sync::CancellationToken;
use zeroize::{Zeroize, Zeroizing};

use super::types::CallbackResult;

const TIMEOUT: Duration = Duration::from_secs(300);
const CONNECTION_TIMEOUT: Duration = Duration::from_secs(5);
const MAX_REQUEST_LEN: usize = 4096;
const MAX_ATTEMPTS: usize = 50;

const SUCCESS_HTML: &str = r#"<!DOCTYPE html>
<html><head><meta charset="utf-8"><title>Beaver</title>
<style>body{font-family:system-ui;display:flex;justify-content:center;
align-items:center;height:100vh;margin:0;background:#1a1a2e;color:#e0e0e0}
.c{text-align:center}h1{color:#f97316}p{margin-top:8px;opacity:.7}</style>
</head><body><div class="c"><h1>Authentification en cours</h1>
<p>Vous pouvez fermer cet onglet et retourner dans l'application.</p>
</div></body></html>"#;

pub struct CallbackServer {
    listener: TcpListener,
    port: u16,
}

impl CallbackServer {
    pub async fn bind() -> Result<Self, String> {
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .map_err(|_| "callback OAuth indisponible".to_string())?;
        let port = listener
            .local_addr()
            .map_err(|_| "callback OAuth indisponible".to_string())?
            .port();
        Ok(Self { listener, port })
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub async fn wait(
        self,
        expected_state: &str,
        cancel: &CancellationToken,
    ) -> Result<CallbackResult, String> {
        tokio::select! {
            result = accept_callback(&self.listener, expected_state) => result,
            _ = tokio::time::sleep(TIMEOUT) => Err("délai d'attente dépassé".to_string()),
            _ = cancel.cancelled() => Err("annulé".to_string()),
        }
    }
}

async fn accept_callback(
    listener: &TcpListener,
    expected_state: &str,
) -> Result<CallbackResult, String> {
    let mut handlers = JoinSet::new();
    let mut accepted = 0usize;
    loop {
        if accepted >= MAX_ATTEMPTS && handlers.is_empty() {
            return Err("trop de requêtes sans callback valide".to_string());
        }
        tokio::select! {
            accepted_stream = listener.accept(), if accepted < MAX_ATTEMPTS => {
                let (mut stream, _) = accepted_stream
                    .map_err(|_| "callback OAuth indisponible".to_string())?;
                let state = Zeroizing::new(expected_state.to_string());
                accepted += 1;
                handlers.spawn(async move {
                    tokio::time::timeout(
                        CONNECTION_TIMEOUT,
                        handle_connection(&mut stream, state.as_str()),
                    )
                    .await
                    .ok()
                    .flatten()
                });
            }
            handled = handlers.join_next(), if !handlers.is_empty() => {
                if let Some(Ok(Some(result))) = handled {
                    return Ok(result);
                }
            }
        }
    }
}

async fn handle_connection(stream: &mut TcpStream, expected_state: &str) -> Option<CallbackResult> {
    let mut buf = Zeroizing::new(vec![0u8; MAX_REQUEST_LEN]);
    let n = stream.read(buf.as_mut_slice()).await.ok()?;

    let parsed = {
        let request = String::from_utf8_lossy(&buf[..n]);
        let first_line = request.lines().next().unwrap_or("");
        parse_callback(first_line)
    };
    buf.zeroize();

    if let Some(result) = parsed.filter(|result| {
        super::flow_auth::verify_state_constant_time(expected_state, result.state.as_str()).is_ok()
    }) {
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\n\
             Content-Length: {}\r\nConnection: close\r\n\r\n{}",
            SUCCESS_HTML.len(),
            SUCCESS_HTML
        );
        let _ = stream.write_all(response.as_bytes()).await;
        let _ = stream.shutdown().await;
        return Some(result);
    }

    let body = "not found";
    let resp_404 = format!(
        "HTTP/1.1 404 Not Found\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    let _ = stream.write_all(resp_404.as_bytes()).await;
    let _ = stream.shutdown().await;
    None
}

fn parse_callback(request_line: &str) -> Option<CallbackResult> {
    let path_and_query = request_line
        .strip_prefix("GET ")?
        .split_whitespace()
        .next()?;

    if !path_and_query.starts_with("/callback?") {
        return None;
    }

    let query = path_and_query.strip_prefix("/callback?")?;
    let mut code: Option<Zeroizing<String>> = None;
    let mut state: Option<Zeroizing<String>> = None;

    for pair in query.split('&') {
        if let Some((k, v)) = pair.split_once('=') {
            match k {
                "code" => code = Some(Zeroizing::new(urldecode(v))),
                "state" => state = Some(Zeroizing::new(urldecode(v))),
                _ => {}
            }
        }
    }

    Some(CallbackResult {
        code: code?,
        state: state?,
    })
}

fn urldecode(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.bytes();
    while let Some(b) = chars.next() {
        match b {
            b'%' => {
                let hi = chars.next().and_then(hex_val);
                let lo = chars.next().and_then(hex_val);
                if let (Some(h), Some(l)) = (hi, lo) {
                    result.push(char::from(h << 4 | l));
                }
            }
            b'+' => result.push(' '),
            _ => result.push(char::from(b)),
        }
    }
    result
}

fn hex_val(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

#[cfg(test)]
#[path = "callback_server_tests.rs"]
mod tests;
