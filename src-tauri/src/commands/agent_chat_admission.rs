use super::agent_chat_streams::StreamEntry;
use crate::ActiveStreams;
use std::future::Future;
use std::sync::Arc;
use tauri::Manager;
use tokio_util::sync::CancellationToken;

pub(crate) struct AgentChatAdmission {
    pub cancel: CancellationToken,
    pub generation: u64,
    pub parent_message_inbox:
        Arc<crate::services::agent_local::parent_message_inbox::ParentMessageInbox>,
    pub permission_mode: String,
    pub request_id: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BackgroundAdmissionError {
    Busy,
    Unavailable,
}

pub(crate) async fn admit_background_if_idle<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
    session_id: &str,
) -> Result<AgentChatAdmission, BackgroundAdmissionError> {
    crate::services::agent_local::session_user_write::ensure_allowed(session_id)
        .await
        .map_err(|_| BackgroundAdmissionError::Unavailable)?;
    let permission_mode = crate::services::agent_local::session_permission_state::prepare_send(
        session_id,
        Some("auto"),
    )
    .await
    .map_err(|_| BackgroundAdmissionError::Unavailable)?;
    let streams = app.state::<ActiveStreams>();
    let session_guard =
        crate::services::agent_local::session_locks::acquire_admission_lease(session_id).await;
    {
        let map = streams.0.lock().await;
        if map.contains_key(session_id) {
            return Err(BackgroundAdmissionError::Busy);
        }
        if map.len()
            >= crate::services::agent_local::agent_work_supervision::MAX_ACTIVE_AGENT_STREAMS
        {
            return Err(BackgroundAdmissionError::Unavailable);
        }
    }
    let cancel = CancellationToken::new();
    let parent_message_inbox =
        Arc::new(crate::services::agent_local::parent_message_inbox::ParentMessageInbox::new());
    let generation = crate::services::agent_local::stream_events::next_generation();
    let request_id =
        crate::services::agent_local::stream_diagnostics::start_request(session_id, generation)
            .await;
    let inserted = {
        let mut map = streams.0.lock().await;
        if map.contains_key(session_id)
            || map.len()
                >= crate::services::agent_local::agent_work_supervision::MAX_ACTIVE_AGENT_STREAMS
        {
            false
        } else {
            map.insert(
                session_id.to_string(),
                (
                    cancel.clone(),
                    generation,
                    request_id.clone(),
                    parent_message_inbox.clone(),
                ),
            );
            true
        }
    };
    if !inserted {
        drop(session_guard);
        crate::services::agent_local::stream_diagnostics::record_failure(
            session_id,
            Some(&request_id),
            "conversation_admission_failed",
            false,
        )
        .await;
        return Err(BackgroundAdmissionError::Unavailable);
    }
    drop(session_guard);
    crate::services::agent_local::subagent_registry::adopt_children_for_parent_stream(
        session_id, &cancel,
    )
    .await;
    Ok(AgentChatAdmission {
        cancel,
        generation,
        parent_message_inbox,
        permission_mode,
        request_id,
    })
}

pub(crate) async fn admit_background(
    app: &tauri::AppHandle,
    session_id: &str,
) -> Result<AgentChatAdmission, String> {
    let streams = app.state::<ActiveStreams>();
    let cancelled_session = session_id.to_string();
    let diagnostic_session = session_id.to_string();
    admit(
        session_id,
        Some("auto"),
        &streams,
        move |(token, _, request_id, inbox)| async move {
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

pub(crate) async fn admit<Cancel, CancelFuture, Start, StartFuture>(
    session_id: &str,
    requested_permission: Option<&str>,
    streams: &ActiveStreams,
    cancel_previous: Cancel,
    start_request: Start,
) -> Result<AgentChatAdmission, String>
where
    Cancel: FnOnce(StreamEntry) -> CancelFuture,
    CancelFuture: Future<Output = ()>,
    Start: FnOnce(u64) -> StartFuture,
    StartFuture: Future<Output = String>,
{
    crate::services::agent_local::session_user_write::ensure_allowed(session_id).await?;
    let permission_mode = crate::services::agent_local::session_permission_state::prepare_send(
        session_id,
        requested_permission,
    )
    .await?;
    let cancel = CancellationToken::new();
    let parent_message_inbox =
        Arc::new(crate::services::agent_local::parent_message_inbox::ParentMessageInbox::new());
    let generation = crate::services::agent_local::stream_events::next_generation();
    let request_id = super::agent_chat_streams::replace_active_stream(
        streams,
        session_id,
        cancel.clone(),
        generation,
        parent_message_inbox.clone(),
        cancel_previous,
        move || start_request(generation),
    )
    .await?;

    Ok(AgentChatAdmission {
        cancel,
        generation,
        parent_message_inbox,
        permission_mode,
        request_id,
    })
}
