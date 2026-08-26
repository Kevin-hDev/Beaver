use super::agent_chat_task::{
    run_stream_task, StreamCapabilityHints, StreamConversation, StreamTaskParams,
};
use crate::models::agent_turn_contract::ChatStreamAdmission;
use crate::services::agent_local::agent_work_supervision::AgentWorkServices;
use crate::services::agent_local::stream_events::AgentEventEmitter;
use crate::services::agent_local::types_ollama::StreamEvent;
use crate::ActiveStreams;
use tauri::Manager;

#[path = "agent_chat_request.rs"]
mod request;
use request::generic_error;
pub(crate) use request::ChatStreamRequest;

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
    #[cfg(debug_assertions)]
    let target_result = match request.fixture_run.as_ref() {
        Some(fixture_run) => {
            super::agent_chat_fixture_candidate::resolve(
                &request.session_id,
                &request.provider,
                &request.model,
                request.reasoning_mode.as_deref(),
                request.capability_hints.supports_thinking,
                fixture_run,
            )
            .await
        }
        None => {
            super::agent_chat_target::resolve(
                &request.session_id,
                &request.provider,
                &request.model,
                request.reasoning_mode.as_deref(),
                request.capability_hints.supports_thinking,
            )
            .await
        }
    };
    #[cfg(not(debug_assertions))]
    let target_result = super::agent_chat_target::resolve(
        &request.session_id,
        &request.provider,
        &request.model,
        request.reasoning_mode.as_deref(),
        request.capability_hints.supports_thinking,
    )
    .await;
    let target = match target_result {
        Ok(target) => target,
        Err(error) => {
            rollback(streams, &request.session_id, &stream).await;
            return Err(error);
        }
    };
    #[cfg(debug_assertions)]
    if target.continuation.is_fixture_candidate() {
        crate::services::agent_local::stream_diagnostics::record_reasoning(
            &request.session_id,
            &stream.request_id,
            "reasoning validation_candidate=true activation=disabled",
        )
        .await;
    }
    request.think = target.reasoning.active;
    request.reasoning_mode = target.reasoning.mode_name.clone();
    request.capability_hints = StreamCapabilityHints::default();
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
    let resolved_dir = match super::agent_working_dir::resolve_for_session(
        &request.session_id,
        request.working_dir.as_deref(),
    )
    .await
    {
        Ok(directory) => directory,
        Err(_) => {
            let admission_rollback = admitted.rollback();
            let rollback_result = super::agent_chat_turn::rollback_current(
                streams,
                &request.session_id,
                stream.generation,
                &admission_rollback,
            )
            .await;
            rollback(streams, &request.session_id, &stream).await;
            return rollback_result.and(Err(generic_error()));
        }
    };
    let result = ChatStreamAdmission {
        generation: stream.generation,
        turn_id: admitted.turn.turn_id.clone(),
        user_message_id: admitted.turn.user_message_id.clone(),
        assistant_message_id: admitted.turn.assistant_message_id.clone(),
    };
    let admission_rollback = admitted.rollback();
    spawn(
        app,
        request,
        streams,
        stream,
        work,
        admitted.turn,
        admission_rollback,
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

include!("agent_chat_run_spawn.rs");
