use super::agent_chat_task::{
    run_stream_task, StreamCapabilityHints, StreamConversation, StreamTaskParams,
};
use crate::models::agent_turn_contract::{ChatStreamAdmission, TurnStart};
use crate::services::agent_local::agent_work_supervision::AgentWorkServices;
use crate::services::agent_local::stream_events::AgentEventEmitter;
use crate::services::agent_local::types_ollama::StreamEvent;
use crate::ActiveStreams;
use tauri::Manager;

pub(crate) struct ChatStreamRequest {
    pub session_id: String,
    pub model: String,
    pub turn: Option<TurnStart>,
    pub tools: Vec<serde_json::Value>,
    pub think: bool,
    pub provider: String,
    pub working_dir: Option<String>,
    pub capability_hints: StreamCapabilityHints,
    pub reasoning_mode: Option<String>,
    pub permission_mode: Option<String>,
    pub plan_mode: Option<bool>,
    #[cfg(debug_assertions)]
    pub fixture_run: Option<crate::services::reasoning_fixture_run::FixtureRunContext>,
}

impl ChatStreamRequest {
    pub(crate) fn from_input(
        input: crate::models::agent_turn_contract::ChatStreamRequestInput,
    ) -> Self {
        Self {
            session_id: input.session_id,
            model: input.model,
            turn: Some(input.turn),
            tools: Vec::new(),
            think: false,
            provider: input.provider,
            working_dir: input.working_dir,
            capability_hints: StreamCapabilityHints::default(),
            reasoning_mode: None,
            permission_mode: input.permission_mode,
            plan_mode: input.plan_mode,
            #[cfg(debug_assertions)]
            fixture_run: None,
        }
    }
}

#[cfg(debug_assertions)]
pub(crate) async fn start_fixture(
    app: tauri::AppHandle,
    mut request: ChatStreamRequest,
    streams: &ActiveStreams,
) -> Result<ChatStreamAdmission, String> {
    request.fixture_run = Some(
        crate::services::reasoning_fixture_run::FixtureRunContext::start()
            .await
            .map_err(|_| generic_error())?,
    );
    start(app, request, streams).await
}

pub(crate) async fn start(
    app: tauri::AppHandle,
    mut request: ChatStreamRequest,
    streams: &ActiveStreams,
) -> Result<ChatStreamAdmission, String> {
    crate::services::agent_local::session_user_write::ensure_allowed(&request.session_id).await?;
    let stream = admit_stream(&app, &request, streams).await?;
    let work = match super::agent_chat_work::admit(&app.state::<AgentWorkServices>()) {
        Ok(work) => work,
        Err(error) => {
            rollback(streams, &request.session_id, &stream).await;
            return Err(error);
        }
    };
    let target = match super::agent_chat_target::resolve(
        &request.session_id,
        &request.provider,
        &request.model,
        request.reasoning_mode.as_deref(),
        request.capability_hints.supports_thinking,
    )
    .await
    {
        Ok(target) => target,
        Err(error) => {
            rollback(streams, &request.session_id, &stream).await;
            return Err(error);
        }
    };
    request.think = target.reasoning.active;
    request.reasoning_mode = target.reasoning.mode_name.clone();
    request.capability_hints = StreamCapabilityHints::default();
    let resolved_dir = match super::agent_working_dir::resolve_for_session(
        &request.session_id,
        request.working_dir.as_deref(),
    )
    .await
    {
        Ok(directory) => directory,
        Err(_) => {
            rollback(streams, &request.session_id, &stream).await;
            return Err(generic_error());
        }
    };
    let turn = match super::agent_chat_turn::prepare(request.turn.take().ok_or_else(generic_error)?)
        .await
    {
        Ok(turn) => turn,
        Err(error) => {
            rollback(streams, &request.session_id, &stream).await;
            return Err(error);
        }
    };
    let admitted = match super::agent_chat_turn::admit_current(
        streams,
        &request.session_id,
        stream.generation,
        turn,
        target.continuation.clone(),
        target.session_reasoning.clone(),
    )
    .await
    {
        Ok(admitted) => admitted,
        Err(error) => {
            rollback(streams, &request.session_id, &stream).await;
            return Err(error);
        }
    };
    let result = ChatStreamAdmission {
        generation: stream.generation,
        turn_id: admitted.turn_id.clone(),
        user_message_id: admitted.user_message_id.clone(),
        assistant_message_id: admitted.assistant_message_id.clone(),
    };
    spawn(
        app,
        request,
        streams,
        stream,
        work,
        admitted,
        target,
        resolved_dir,
        result.clone(),
    )
    .await?;
    Ok(result)
}

async fn admit_stream(
    app: &tauri::AppHandle,
    request: &ChatStreamRequest,
    streams: &ActiveStreams,
) -> Result<super::agent_chat_admission::AgentChatAdmission, String> {
    let replacement_app = app.clone();
    let cancelled_session = request.session_id.clone();
    let diagnostic_session = request.session_id.clone();
    super::agent_chat_admission::admit(
        &request.session_id,
        request.permission_mode.as_deref(),
        streams,
        move |(token, generation, request_id, inbox)| async move {
            crate::services::mascot::cancel_session(
                &replacement_app,
                &cancelled_session,
                generation,
            );
            inbox.close().await;
            crate::services::agent_local::session_locks::cancel_with_lock(
                &cancelled_session,
                &token,
            )
            .await;
            crate::services::agent_local::stream_diagnostics::record_cancelled(
                &cancelled_session,
                &request_id,
            )
            .await;
        },
        move |generation| async move {
            crate::services::agent_local::stream_diagnostics::start_request(
                &diagnostic_session,
                generation,
            )
            .await
        },
    )
    .await
}

pub(crate) async fn rollback(
    streams: &ActiveStreams,
    session_id: &str,
    stream: &super::agent_chat_admission::AgentChatAdmission,
) {
    stream.cancel.cancel();
    stream.parent_message_inbox.close().await;
    let mut map = streams.0.lock().await;
    let current = matches!(map.get(session_id), Some((_, generation, _, _)) if *generation == stream.generation);
    if current {
        map.remove(session_id);
    }
    drop(map);
    if current {
        crate::services::agent_local::stream_diagnostics::record_failure(
            session_id,
            Some(&stream.request_id),
            generic_error().as_str(),
            false,
        )
        .await;
    }
}

fn generic_error() -> String {
    "conversation_admission_failed".to_string()
}

include!("agent_chat_run_spawn.rs");
