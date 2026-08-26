use crate::ActiveStreams;
use std::future::Future;
use std::sync::Arc;
use tokio_util::sync::CancellationToken;

pub(crate) const ACTIVE_STREAM_LIMIT_REACHED: &str = "active-stream-limit-reached";
pub(crate) const STREAM_REPLACED: &str = "stream-replaced";

pub(crate) type StreamEntry = (
    CancellationToken,
    u64,
    String,
    Arc<crate::services::agent_local::parent_message_inbox::ParentMessageInbox>,
);

pub(crate) async fn replace_active_stream<Cancel, CancelFuture, Start, StartFuture>(
    streams: &ActiveStreams,
    session_id: &str,
    cancel: CancellationToken,
    generation: u64,
    inbox: Arc<crate::services::agent_local::parent_message_inbox::ParentMessageInbox>,
    cancel_previous: Cancel,
    start_request: Start,
) -> Result<String, String>
where
    Cancel: FnOnce(StreamEntry) -> CancelFuture,
    CancelFuture: Future<Output = ()>,
    Start: FnOnce() -> StartFuture,
    StartFuture: Future<Output = String>,
{
    {
        let map = streams.0.lock().await;
        if map.len()
            >= crate::services::agent_local::agent_work_supervision::MAX_ACTIVE_AGENT_STREAMS
            && !map.contains_key(session_id)
        {
            return Err(ACTIVE_STREAM_LIMIT_REACHED.to_string());
        }
    }
    let request_id = start_request().await;
    // Même lease que l'admission durable : elle définit l'ordre remplacement/admission.
    let session_lease = crate::services::agent_local::session_store::lock_session(session_id).await;
    let session_guard = session_lease.lock().await;
    let inserted = {
        let mut map = streams.0.lock().await;
        if map.len()
            >= crate::services::agent_local::agent_work_supervision::MAX_ACTIVE_AGENT_STREAMS
            && !map.contains_key(session_id)
        {
            None
        } else {
            Some(map.insert(
                session_id.to_string(),
                (cancel.clone(), generation, request_id.clone(), inbox),
            ))
        }
    };
    let Some(old_stream) = inserted else {
        drop(session_guard);
        crate::services::agent_local::stream_diagnostics::record_failure(
            session_id,
            Some(&request_id),
            "conversation_admission_failed",
            false,
        )
        .await;
        return Err(ACTIVE_STREAM_LIMIT_REACHED.to_string());
    };
    drop(session_guard);
    crate::services::agent_local::subagent_registry::adopt_children_for_parent_stream(
        session_id, &cancel,
    )
    .await;
    if let Some(old_stream) = old_stream {
        cancel_previous(old_stream).await;
    }
    let is_current = matches!(
        streams.0.lock().await.get(session_id),
        Some((_, active_generation, _, _)) if *active_generation == generation
    );
    if !is_current {
        cancel.cancel();
        return Err(STREAM_REPLACED.to_string());
    }
    Ok(request_id)
}
