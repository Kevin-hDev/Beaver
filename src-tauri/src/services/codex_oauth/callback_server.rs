use std::time::Duration;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::task::JoinSet;
use tokio_util::sync::CancellationToken;
use zeroize::Zeroizing;

use super::callback::{self, CallbackError, CallbackResult};

const CALLBACK_PORTS: &[u16] = &[1455, 1457];
const TIMEOUT: Duration = Duration::from_secs(120);
const CONNECTION_TIMEOUT: Duration = Duration::from_secs(5);
const MAX_CALLBACK_ATTEMPTS: usize = 16;

pub(super) struct CallbackServer {
    listener: TcpListener,
    port: u16,
}

impl CallbackServer {
    pub(super) async fn bind() -> Result<Self, String> {
        let listener = bind_first_available(CALLBACK_PORTS).await?;
        let port = listener.local_addr().map_err(|_| unavailable())?.port();
        Ok(Self { listener, port })
    }

    pub(super) fn redirect_uri(&self) -> String {
        format!("http://localhost:{}/auth/callback", self.port)
    }

    pub(super) async fn wait(
        self,
        expected_state: &str,
        cancel: &CancellationToken,
    ) -> Result<CallbackResult, String> {
        callback::validate_state(expected_state)?;
        tokio::time::timeout(TIMEOUT, async {
            tokio::select! {
                result = accept_until_valid(&self.listener, expected_state) => result,
                _ = cancel.cancelled() => Err("callback OAuth annulé".to_string()),
            }
        })
        .await
        .map_err(|_| "callback OAuth expiré".to_string())?
    }
}

async fn bind_first_available(ports: &[u16]) -> Result<TcpListener, String> {
    for port in ports.iter().copied() {
        if let Ok(listener) = TcpListener::bind(("127.0.0.1", port)).await {
            return Ok(listener);
        }
    }
    Err(unavailable())
}

async fn accept_until_valid(
    listener: &TcpListener,
    expected_state: &str,
) -> Result<CallbackResult, String> {
    let mut handlers = JoinSet::new();
    let mut accepted = 0_usize;
    loop {
        if accepted >= MAX_CALLBACK_ATTEMPTS && handlers.is_empty() {
            return Err(unavailable());
        }
        tokio::select! {
            accepted_stream = listener.accept(), if accepted < MAX_CALLBACK_ATTEMPTS => {
                let (mut stream, _) = accepted_stream.map_err(|_| unavailable())?;
                let state = Zeroizing::new(expected_state.to_string());
                accepted += 1;
                handlers.spawn(async move {
                    tokio::time::timeout(
                        CONNECTION_TIMEOUT,
                        handle_connection(&mut stream, state.as_str()),
                    )
                    .await
                    .unwrap_or(Err(CallbackError::Invalid))
                });
            }
            handled = handlers.join_next(), if !handlers.is_empty() => {
                match handled {
                    Some(Ok(Ok(result))) => return Ok(result),
                    Some(Ok(Err(CallbackError::Refused))) => {
                        return Err("callback OAuth refusé".to_string());
                    }
                    Some(Ok(Err(CallbackError::Invalid)) | Err(_)) => {}
                    None => {}
                }
            }
        }
    }
}

async fn handle_connection(
    stream: &mut TcpStream,
    expected_state: &str,
) -> Result<CallbackResult, CallbackError> {
    let result = read_request(stream)
        .await
        .and_then(|request| callback::parse_callback_bytes(&request, expected_state));
    let (status, body) = if result.is_ok() {
        (
            "200 OK",
            "Connexion réussie. Vous pouvez fermer cet onglet.",
        )
    } else {
        ("400 Bad Request", "Requête OAuth invalide.")
    };
    let response = format!(
        "HTTP/1.1 {status}\r\nContent-Type: text/plain; charset=utf-8\r\nCache-Control: no-store\r\nX-Content-Type-Options: nosniff\r\nConnection: close\r\nContent-Length: {}\r\n\r\n{body}",
        body.len()
    );
    let _ = stream.write_all(response.as_bytes()).await;
    let _ = stream.shutdown().await;
    result
}

async fn read_request(stream: &mut TcpStream) -> Result<Zeroizing<Vec<u8>>, CallbackError> {
    let mut request = Zeroizing::new(Vec::with_capacity(1024));
    let mut chunk = [0_u8; 1024];
    loop {
        let read = stream
            .read(&mut chunk)
            .await
            .map_err(|_| CallbackError::Invalid)?;
        if read == 0 || request.len().saturating_add(read) > callback::MAX_REQUEST_BYTES {
            return Err(CallbackError::Invalid);
        }
        request.extend_from_slice(&chunk[..read]);
        if request.windows(4).any(|window| window == b"\r\n\r\n") {
            return Ok(request);
        }
    }
}

fn unavailable() -> String {
    "callback OAuth indisponible".to_string()
}

#[cfg(test)]
#[path = "callback_server_tests.rs"]
mod tests;
