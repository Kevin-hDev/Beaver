use crate::ActiveStreams;

#[tauri::command]
pub async fn cancel_agent_request(
    app: tauri::AppHandle,
    session_id: String,
    generation: Option<u64>,
    streams: tauri::State<'_, ActiveStreams>,
) -> Result<(), String> {
    let mut cancelled = false;
    let mut map = streams.0.lock().await;
    if let Some((token, gen, request_id, inbox)) = map.get(&session_id) {
        if generation.is_none() || generation == Some(*gen) {
            let token = token.clone();
            let request_id = request_id.clone();
            let inbox = inbox.clone();
            map.remove(&session_id);
            drop(map);
            inbox.close().await;
            crate::services::agent_local::session_locks::cancel_with_lock(&session_id, &token)
                .await;
            crate::services::agent_local::stream_diagnostics::record_cancelled(
                &session_id,
                &request_id,
            )
            .await;
            cancelled = true;
        }
    }
    if crate::services::agent_local::subagent_cancellation::cancel(&session_id)
        .await
        .unwrap_or(false)
    {
        cancelled = true;
    }
    if cancelled {
        crate::services::mascot::end_session(&app, &session_id);
        crate::services::agent_local::subagent_registry::cancel_stopped_parent_stream_children(
            &session_id,
        )
        .await;
    }
    Ok(())
}
