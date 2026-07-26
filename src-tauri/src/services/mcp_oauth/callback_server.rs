use std::time::Duration;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::sync::oneshot;
use tokio_util::sync::CancellationToken;
use zeroize::{Zeroize, Zeroizing};

use super::types::CallbackResult;

const TIMEOUT: Duration = Duration::from_secs(300);
const MAX_REQUEST_LEN: usize = 4096;

const SUCCESS_HTML: &str = r#"<!DOCTYPE html>
<html><head><meta charset="utf-8"><title>Beaver</title>
<style>body{font-family:system-ui;display:flex;justify-content:center;
align-items:center;height:100vh;margin:0;background:#1a1a2e;color:#e0e0e0}
.c{text-align:center}h1{color:#f97316}p{margin-top:8px;opacity:.7}</style>
</head><body><div class="c"><h1>Authentification en cours</h1>
<p>Vous pouvez fermer cet onglet et retourner dans l'application.</p>
</div></body></html>"#;

pub async fn start(
    cancel: CancellationToken,
) -> Result<(u16, oneshot::Receiver<Result<CallbackResult, String>>), String> {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .map_err(|e| format!("bind: {e}"))?;

    let port = listener
        .local_addr()
        .map_err(|e| format!("addr: {e}"))?
        .port();

    let (tx, rx) = oneshot::channel();

    tokio::spawn(async move {
        let result = tokio::select! {
            r = accept_callback(&listener) => r,
            () = tokio::time::sleep(TIMEOUT) => Err("délai d'attente dépassé".to_string()),
            () = cancel.cancelled() => Err("annulé".to_string()),
        };
        let _ = tx.send(result);
    });

    Ok((port, rx))
}

async fn accept_callback(listener: &TcpListener) -> Result<CallbackResult, String> {
    let mut attempts = 0u32;
    const MAX_ATTEMPTS: u32 = 50;
    loop {
        attempts += 1;
        if attempts > MAX_ATTEMPTS {
            return Err("trop de requêtes sans callback valide".to_string());
        }
        let (mut stream, _) = listener
            .accept()
            .await
            .map_err(|e| format!("accept: {e}"))?;

        let mut buf = Zeroizing::new(vec![0u8; MAX_REQUEST_LEN]);
        let n = stream
            .read(buf.as_mut_slice())
            .await
            .map_err(|e| format!("read: {e}"))?;

        let parsed = {
            let request = String::from_utf8_lossy(&buf[..n]);
            let first_line = request.lines().next().unwrap_or("");
            parse_callback(first_line)
        };
        buf.zeroize();

        if let Some(result) = parsed {
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\n\
                 Content-Length: {}\r\nConnection: close\r\n\r\n{}",
                SUCCESS_HTML.len(),
                SUCCESS_HTML
            );
            let _ = stream.write_all(response.as_bytes()).await;
            let _ = stream.shutdown().await;
            return Ok(result);
        }

        let body = "not found";
        let resp_404 = format!(
            "HTTP/1.1 404 Not Found\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        );
        let _ = stream.write_all(resp_404.as_bytes()).await;
        let _ = stream.shutdown().await;
    }
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
