#[path = "test_transport/http.rs"]
mod http;
#[path = "test_transport/websocket.rs"]
mod websocket;

use std::sync::{LazyLock, Mutex, MutexGuard};
use tokio::sync::{Mutex as AsyncMutex, OwnedMutexGuard};

const MAX_SCRIPT_ITEMS: usize = 16;
const MAX_CAPTURES: usize = 16;

#[derive(Debug, Clone, Copy)]
pub(crate) enum HttpReply {
    Unauthorized,
    Success,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum WebSocketReply {
    Success,
    Unavailable,
}

#[derive(Debug, Clone)]
pub(crate) struct HttpCapture {
    pub body: serde_json::Value,
    pub routing_hint: Option<String>,
    pub authorization_valid: bool,
    pub account_header_present: bool,
    pub originator_valid: bool,
    pub user_agent_present: bool,
    pub path: String,
    pub body_has_access_token: bool,
    pub body_bytes: usize,
}

#[derive(Debug, Clone)]
pub(crate) struct WebSocketCapture {
    pub body: serde_json::Value,
    pub routing_hint: Option<String>,
    pub authorization_valid: bool,
    pub account_header_present: bool,
    pub originator_valid: bool,
    pub user_agent_present: bool,
    pub beta_header_valid: bool,
    pub session_headers_valid: bool,
    pub body_has_access_token: bool,
    pub body_bytes: usize,
}

#[derive(Default)]
struct State {
    active: bool,
    http_script: Option<Vec<HttpReply>>,
    websocket_script: Option<WebSocketReply>,
    http_captures: Vec<HttpCapture>,
    websocket_captures: Vec<WebSocketCapture>,
    refresh_count: usize,
}

static SERIAL: LazyLock<std::sync::Arc<AsyncMutex<()>>> =
    LazyLock::new(|| std::sync::Arc::new(AsyncMutex::new(())));
static STATE: LazyLock<Mutex<State>> = LazyLock::new(|| Mutex::new(State::default()));
static WEBSOCKET_CAPTURED: LazyLock<tokio::sync::Notify> = LazyLock::new(tokio::sync::Notify::new);

pub(super) async fn dispatch_http(
    body: &str,
    routing_hint: &str,
    model: &str,
    tool_count: usize,
) -> Option<Result<reqwest::Response, String>> {
    http::dispatch_http(body, routing_hint, model, tool_count).await
}

pub(super) async fn connect_websocket(
    session_id: &str,
    routing_hint: &str,
) -> Option<Result<super::websocket_connect::CodexSocket, super::websocket_connect::ConnectError>> {
    websocket::connect_websocket(session_id, routing_hint).await
}

pub(crate) struct CodexTransportScenario {
    _serial: OwnedMutexGuard<()>,
}

impl CodexTransportScenario {
    pub(crate) async fn start(
        http_script: Option<Vec<HttpReply>>,
        websocket_script: Option<WebSocketReply>,
    ) -> Self {
        let serial = SERIAL.clone().lock_owned().await;
        if let Some(script) = &http_script {
            assert!(!script.is_empty());
            assert!(script.len() <= MAX_SCRIPT_ITEMS);
        }
        *state() = State {
            active: true,
            http_script,
            websocket_script,
            ..State::default()
        };
        super::websocket::mark_available();
        Self { _serial: serial }
    }

    pub(crate) fn http_captures(&self) -> Vec<HttpCapture> {
        state().http_captures.clone()
    }

    pub(crate) fn websocket_captures(&self) -> Vec<WebSocketCapture> {
        state().websocket_captures.clone()
    }

    pub(crate) fn refresh_count(&self) -> usize {
        state().refresh_count
    }

    pub(crate) async fn wait_for_websocket_captures(&self, expected: usize) {
        let wait = async {
            loop {
                let notified = WEBSOCKET_CAPTURED.notified();
                if state().websocket_captures.len() >= expected {
                    return;
                }
                notified.await;
            }
        };
        tokio::time::timeout(std::time::Duration::from_secs(2), wait)
            .await
            .expect("bounded WebSocket capture wait");
    }
}

impl Drop for CodexTransportScenario {
    fn drop(&mut self) {
        *state() = State::default();
        super::websocket::mark_available();
    }
}

fn state() -> MutexGuard<'static, State> {
    STATE.lock().unwrap_or_else(|error| error.into_inner())
}

fn record_http(capture: HttpCapture) -> Result<(), String> {
    let mut state = state();
    if state.http_captures.len() >= MAX_CAPTURES {
        return Err("provider_configuration_invalid".to_string());
    }
    state.http_captures.push(capture);
    Ok(())
}

fn record_websocket(capture: WebSocketCapture) -> Result<(), String> {
    let mut state = state();
    if state.websocket_captures.len() >= MAX_CAPTURES {
        return Err("provider_configuration_invalid".to_string());
    }
    state.websocket_captures.push(capture);
    drop(state);
    WEBSOCKET_CAPTURED.notify_waiters();
    Ok(())
}

fn record_refresh() {
    let mut state = state();
    state.refresh_count = state.refresh_count.saturating_add(1);
}
