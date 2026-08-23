#[path = "test_transport/http.rs"]
mod http;
#[path = "test_transport/projection.rs"]
mod projection;
#[path = "test_transport/response.rs"]
mod response;
#[path = "test_transport/sensitive_buffer.rs"]
mod sensitive_buffer;
#[path = "test_transport/types.rs"]
mod types;
#[path = "test_transport/websocket.rs"]
mod websocket;
#[path = "test_transport/websocket_raw.rs"]
mod websocket_raw;

use std::future::Future;
use std::sync::{Arc, LazyLock, Mutex, MutexGuard};
use tokio::sync::{Mutex as AsyncMutex, OwnedMutexGuard};
pub(crate) use types::{
    HttpCapture, HttpReply, RequestProjection, WebSocketCapture, WebSocketReply,
};

const MAX_SCRIPT_ITEMS: usize = 16;
const MAX_CAPTURES: usize = 16;

#[derive(Default)]
struct State {
    http_script: Option<Vec<HttpReply>>,
    websocket_script: Option<WebSocketReply>,
    http_captures: Vec<HttpCapture>,
    websocket_captures: Vec<WebSocketCapture>,
    refresh_count: usize,
    initial_credentials_dropped_before_refresh: bool,
    initial_response_dropped_before_refresh: bool,
    rejected_access_valid: bool,
}

struct ScenarioContext {
    state: Mutex<State>,
    websocket_captured: tokio::sync::Notify,
    http_payload_zeroized: Arc<std::sync::atomic::AtomicBool>,
    websocket_payload_zeroized: Arc<std::sync::atomic::AtomicBool>,
    initial_response_dropped: Arc<std::sync::atomic::AtomicBool>,
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

pub(super) fn observe_initial_response(response: reqwest::Response) -> response::ObservedResponse {
    response::observe(response)
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
            http_payload_zeroized: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            websocket_payload_zeroized: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            initial_response_dropped: Arc::new(std::sync::atomic::AtomicBool::new(false)),
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

    pub(crate) fn initial_response_dropped_before_refresh(&self) -> bool {
        state(&self.context).initial_response_dropped_before_refresh
    }

    pub(crate) fn rejected_access_valid(&self) -> bool {
        state(&self.context).rejected_access_valid
    }

    pub(crate) fn http_payload_buffer_zeroized(&self) -> bool {
        self.context
            .http_payload_zeroized
            .load(std::sync::atomic::Ordering::SeqCst)
    }

    pub(crate) fn websocket_payload_buffer_zeroized(&self) -> bool {
        self.context
            .websocket_payload_zeroized
            .load(std::sync::atomic::Ordering::SeqCst)
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

fn record_refresh(
    context: &ScenarioContext,
    initial_credentials_dropped: bool,
    rejected_access_valid: bool,
) {
    let mut state = state(context);
    state.refresh_count = state.refresh_count.saturating_add(1);
    state.initial_credentials_dropped_before_refresh = initial_credentials_dropped;
    state.initial_response_dropped_before_refresh = context
        .initial_response_dropped
        .load(std::sync::atomic::Ordering::SeqCst);
    state.rejected_access_valid = rejected_access_valid;
}
