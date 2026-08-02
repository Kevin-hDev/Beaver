use crate::ActiveStreams;

#[tauri::command]
pub async fn cancel_agent_request(
    app: tauri::AppHandle,
    session_id: String,
    generation: Option<u64>,
    streams: tauri::State<'_, ActiveStreams>,
) -> Result<(), String> {
    let mut cancelled = false;
    let active_stream = {
        let mut map = streams.0.lock().await;
        match map.get(&session_id) {
            Some((token, active_generation, request_id, inbox))
                if generation.is_none() || generation == Some(*active_generation) =>
            {
                let stream = (
                    token.clone(),
                    *active_generation,
                    request_id.clone(),
                    inbox.clone(),
                );
                map.remove(&session_id);
                Some(stream)
            }
            _ => None,
        }
    };
    if let Some(stream) = active_stream {
        cancel_active_stream(&app, &session_id, stream).await;
        cancelled = true;
    }
    if crate::services::agent_local::subagent_cancellation::cancel(&session_id)
        .await
        .unwrap_or(false)
    {
        cancelled = true;
    }
    if cancelled {
        crate::services::agent_local::subagent_registry::cancel_stopped_parent_stream_children(
            &session_id,
        )
        .await;
    }
    Ok(())
}

pub(crate) async fn cancel_all_agent_requests(app: &tauri::AppHandle, streams: &ActiveStreams) {
    let active = {
        let mut map = streams.0.lock().await;
        map.drain().collect::<Vec<_>>()
    };
    futures_util::future::join_all(active.into_iter().map(|(session_id, stream)| async move {
        cancel_active_stream(app, &session_id, stream).await;
        crate::services::agent_local::subagent_registry::cancel_stopped_parent_stream_children(
            &session_id,
        )
        .await;
    }))
    .await;
}

async fn cancel_active_stream(
    app: &tauri::AppHandle,
    session_id: &str,
    stream: super::agent_chat_streams::StreamEntry,
) {
    let (token, generation, request_id, inbox) = stream;
    crate::services::mascot::cancel_session(app, session_id, generation);
    inbox.close().await;
    crate::services::agent_local::session_locks::cancel_with_lock(session_id, &token).await;
    crate::services::agent_local::stream_diagnostics::record_cancelled(session_id, &request_id)
        .await;
}
