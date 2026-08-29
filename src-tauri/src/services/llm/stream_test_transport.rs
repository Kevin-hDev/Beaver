use super::stream_http::RequestError;
use std::collections::VecDeque;
use std::sync::{LazyLock, Mutex, MutexGuard};
use tokio::sync::{Mutex as AsyncMutex, OwnedMutexGuard, Semaphore};

const MAX_SCRIPTED_RESPONSES: usize = 16;
const MAX_RECORDED_PAYLOADS: usize = 16;
const MAX_SCRIPTED_FRAGMENTS: usize = 64;

#[derive(Debug, Clone, Copy)]
pub(crate) enum ScriptedResponse {
    RetryablePaused,
    ServiceTierRejected,
    PayloadTooLargePaused,
    Success,
}

#[derive(Default)]
struct State {
    active: bool,
    session_id: String,
    responses: VecDeque<ScriptedResponse>,
    fragments: Vec<String>,
    payloads: Vec<serde_json::Value>,
}

static SERIAL: LazyLock<std::sync::Arc<AsyncMutex<()>>> =
    LazyLock::new(|| std::sync::Arc::new(AsyncMutex::new(())));
static STATE: LazyLock<Mutex<State>> = LazyLock::new(|| Mutex::new(State::default()));
static RECORDED: LazyLock<tokio::sync::Notify> = LazyLock::new(tokio::sync::Notify::new);
static RELEASE: LazyLock<Semaphore> = LazyLock::new(|| Semaphore::new(0));

pub(crate) struct StreamScenario {
    _serial: OwnedMutexGuard<()>,
}

impl StreamScenario {
    pub(crate) async fn start(
        session_id: &str,
        responses: impl IntoIterator<Item = ScriptedResponse>,
    ) -> Self {
        let serial = SERIAL.clone().lock_owned().await;
        replace_state(session_id, responses, Vec::new());
        Self { _serial: serial }
    }

    pub(crate) async fn start_with_fragments(
        session_id: &str,
        fragments: Vec<String>,
    ) -> Result<Self, &'static str> {
        validate_fragments(&fragments)?;
        let serial = SERIAL.clone().lock_owned().await;
        replace_state(session_id, [ScriptedResponse::Success], fragments);
        Ok(Self { _serial: serial })
    }

    pub(crate) async fn wait_for_payloads(&self, expected: usize) {
        loop {
            let notified = RECORDED.notified();
            if state().payloads.len() >= expected {
                return;
            }
            notified.await;
        }
    }

    pub(crate) fn release_one(&self) {
        RELEASE.add_permits(1);
    }

    pub(crate) fn payloads(&self) -> Vec<serde_json::Value> {
        state().payloads.clone()
    }
}

impl Drop for StreamScenario {
    fn drop(&mut self) {
        *state() = State::default();
        drain_releases();
    }
}

pub(super) async fn dispatch(
    cfg: &super::stream_http::RequestConfig<'_>,
    payload: &serde_json::Value,
) -> Option<Result<reqwest::Response, RequestError>> {
    let (response, fragments) = {
        let mut state = state();
        if !state.active || cfg.session_id != Some(state.session_id.as_str()) {
            return None;
        }
        if state.payloads.len() >= MAX_RECORDED_PAYLOADS {
            return Some(Err(RequestError::InvalidConfiguration));
        }
        state.payloads.push(payload.clone());
        (state.responses.pop_front(), state.fragments.clone())
    };
    RECORDED.notify_waiters();
    match response {
        Some(ScriptedResponse::RetryablePaused) => {
            wait_for_release().await;
            Some(Err(RequestError::Fatal(
                "provider_temporarily_unavailable".into(),
            )))
        }
        Some(ScriptedResponse::ServiceTierRejected) => {
            Some(Err(super::stream_http::classify_error(
                400,
                r#"{"error":{"param":"service_tier","code":"invalid_request_error"}}"#,
                "OpenAI",
                super::route_profile::ErrorPolicy::Responses,
                false,
                false,
            )))
        }
        Some(ScriptedResponse::PayloadTooLargePaused) => {
            wait_for_release().await;
            Some(Err(RequestError::PayloadTooLarge))
        }
        Some(ScriptedResponse::Success) => Some(Ok(success_response(
            matches!(
                super::route_profile::find(cfg.provider_id).map(|profile| profile.wire.family),
                Some(super::route_profile::WireFamily::AnthropicMessages)
            ),
            payload.get("input").is_some(),
            &fragments,
        ))),
        None => Some(Err(RequestError::InvalidConfiguration)),
    }
}

fn replace_state(
    session_id: &str,
    responses: impl IntoIterator<Item = ScriptedResponse>,
    fragments: Vec<String>,
) {
    let responses = responses.into_iter().collect::<VecDeque<_>>();
    assert!(responses.len() <= MAX_SCRIPTED_RESPONSES);
    *state() = State {
        active: true,
        session_id: session_id.to_string(),
        responses,
        fragments,
        payloads: Vec::new(),
    };
    drain_releases();
}

fn state() -> MutexGuard<'static, State> {
    STATE.lock().unwrap_or_else(|error| error.into_inner())
}

async fn wait_for_release() {
    RELEASE
        .acquire()
        .await
        .expect("test release semaphore remains open")
        .forget();
}

fn drain_releases() {
    while let Ok(permit) = RELEASE.try_acquire() {
        permit.forget();
    }
}

fn success_response(
    anthropic_messages: bool,
    responses_api: bool,
    fragments: &[String],
) -> reqwest::Response {
    let body = if !fragments.is_empty() {
        let mut body = fragments
            .iter()
            .map(|fragment| format!("data: {fragment}\n\n"))
            .collect::<String>();
        body.push_str("data: [DONE]\n\n");
        body
    } else if anthropic_messages {
        concat!(
            "data: {\"type\":\"message_start\",\"message\":{\"usage\":{\"input_tokens\":1,\"output_tokens\":0}}}\n\n",
            "data: {\"type\":\"content_block_start\",\"index\":0,\"content_block\":{\"type\":\"text\",\"text\":\"\"}}\n\n",
            "data: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"text_delta\",\"text\":\"ok\"}}\n\n",
            "data: {\"type\":\"content_block_stop\",\"index\":0}\n\n",
            "data: {\"type\":\"message_delta\",\"delta\":{\"stop_reason\":\"end_turn\"},\"usage\":{\"output_tokens\":1}}\n\n",
            "data: {\"type\":\"message_stop\"}\n\n",
        )
        .to_string()
    } else if responses_api {
        concat!(
            "data: {\"type\":\"response.output_text.delta\",\"delta\":\"ok\"}\n\n",
            "data: {\"type\":\"response.completed\",\"response\":{\"usage\":{\"input_tokens\":1,\"output_tokens\":1}}}\n\n",
        )
        .to_string()
    } else {
        concat!(
            "data: {\"choices\":[{\"delta\":{\"content\":\"ok\"},\"finish_reason\":\"stop\"}]}\n\n",
            "data: [DONE]\n\n",
        )
        .to_string()
    };
    let response = tauri::http::Response::builder()
        .status(200)
        .header("content-type", "text/event-stream")
        .body(body)
        .expect("valid scripted response");
    reqwest::Response::from(response)
}

fn validate_fragments(fragments: &[String]) -> Result<(), &'static str> {
    if fragments.len() > MAX_SCRIPTED_FRAGMENTS {
        return Err("too_many_scripted_fragments");
    }
    Ok(())
}

#[tokio::test]
async fn scripted_fragment_list_is_bounded() {
    let fragments = vec![String::new(); MAX_SCRIPTED_FRAGMENTS + 1];
    let result = StreamScenario::start_with_fragments("bounded-fixture", fragments).await;
    assert!(matches!(result, Err("too_many_scripted_fragments")));
}
