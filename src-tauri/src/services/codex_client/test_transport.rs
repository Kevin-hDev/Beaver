#[path = "test_transport/http.rs"]
mod http;
#[path = "test_transport/projection.rs"]
mod projection;
#[path = "test_transport/websocket.rs"]
mod websocket;

use std::future::Future;
use std::sync::{Arc, LazyLock, Mutex, MutexGuard};
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
    pub request: RequestProjection,
    pub routing_hint: Option<String>,
    pub authorization_valid: bool,
    pub account_header_present: bool,
    pub originator_valid: bool,
    pub user_agent_present: bool,
    pub response_path_valid: bool,
}

#[derive(Debug, Clone)]
pub(crate) struct WebSocketCapture {
    pub request: RequestProjection,
    pub routing_hint: Option<String>,
    pub authorization_valid: bool,
    pub account_header_present: bool,
    pub originator_valid: bool,
    pub user_agent_present: bool,
    pub beta_header_valid: bool,
    pub session_headers_valid: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RequestProjection {
    pub model: String,
    pub service_tier: Option<String>,
    pub envelope_type: Option<String>,
    pub input_count: usize,
    pub tool_count: usize,
    pub forbidden_field_present: bool,
    pub body_bytes: usize,
}

#[derive(Default)]
struct State {
    http_script: Option<Vec<HttpReply>>,
    websocket_script: Option<WebSocketReply>,
    http_captures: Vec<HttpCapture>,
    websocket_captures: Vec<WebSocketCapture>,
    refresh_count: usize,
    initial_credentials_dropped_before_refresh: bool,
}

struct ScenarioContext {
    state: Mutex<State>,
    websocket_captured: tokio::sync::Notify,
}

tokio::task_local! {
    static ACTIVE_SCENARIO: Arc<ScenarioContext>;
}

static SERIAL: LazyLock<std::sync::Arc<AsyncMutex<()>>> =
    LazyLock::new(|| std::sync::Arc::new(AsyncMutex::new(())));
pub(super) async fn dispatch_http(
    body: &str,
    routing_hint: &str,
    model: &str,
    tool_count: usize,
) -> Option<Result<reqwest::Response, String>> {
    let context = ACTIVE_SCENARIO.try_with(Arc::clone).ok()?;
    Some(http::dispatch_http(context, body, routing_hint, model, tool_count).await)
}

pub(super) async fn connect_websocket(
    session_id: &str,
    routing_hint: &str,
) -> Option<Result<super::websocket_connect::CodexSocket, super::websocket_connect::ConnectError>> {
    let context = ACTIVE_SCENARIO.try_with(Arc::clone).ok()?;
    Some(websocket::connect_websocket(context, session_id, routing_hint).await)
}

pub(crate) struct CodexTransportScenario {
    _serial: OwnedMutexGuard<()>,
    context: Arc<ScenarioContext>,
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
        let context = Arc::new(ScenarioContext {
            state: Mutex::new(State {
                http_script,
                websocket_script,
                ..State::default()
            }),
            websocket_captured: tokio::sync::Notify::new(),
        });
        super::websocket::mark_available();
        Self {
            _serial: serial,
            context,
        }
    }

    pub(crate) async fn scope<F>(&self, future: F) -> F::Output
    where
        F: Future,
    {
        ACTIVE_SCENARIO
            .scope(Arc::clone(&self.context), future)
            .await
    }

    pub(crate) fn http_captures(&self) -> Vec<HttpCapture> {
        state(&self.context).http_captures.clone()
    }

    pub(crate) fn websocket_captures(&self) -> Vec<WebSocketCapture> {
        state(&self.context).websocket_captures.clone()
    }

    pub(crate) fn refresh_count(&self) -> usize {
        state(&self.context).refresh_count
    }

    pub(crate) fn initial_credentials_dropped_before_refresh(&self) -> bool {
        state(&self.context).initial_credentials_dropped_before_refresh
    }

    pub(crate) async fn wait_for_websocket_captures(&self, expected: usize) {
        let wait = async {
            loop {
                let notified = self.context.websocket_captured.notified();
                if state(&self.context).websocket_captures.len() >= expected {
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
        super::websocket::mark_available();
    }
}

fn state(context: &ScenarioContext) -> MutexGuard<'_, State> {
    context
        .state
        .lock()
        .unwrap_or_else(|error| error.into_inner())
}

fn record_http(context: &ScenarioContext, capture: HttpCapture) -> Result<(), String> {
    let mut state = state(context);
    if state.http_captures.len() >= MAX_CAPTURES {
        return Err("provider_configuration_invalid".to_string());
    }
    state.http_captures.push(capture);
    Ok(())
}

fn record_websocket(context: &ScenarioContext, capture: WebSocketCapture) -> Result<(), String> {
    let mut state = state(context);
    if state.websocket_captures.len() >= MAX_CAPTURES {
        return Err("provider_configuration_invalid".to_string());
    }
    state.websocket_captures.push(capture);
    drop(state);
    context.websocket_captured.notify_waiters();
    Ok(())
}

fn record_refresh(context: &ScenarioContext, initial_credentials_dropped: bool) {
    let mut state = state(context);
    state.refresh_count = state.refresh_count.saturating_add(1);
    state.initial_credentials_dropped_before_refresh = initial_credentials_dropped;
}
