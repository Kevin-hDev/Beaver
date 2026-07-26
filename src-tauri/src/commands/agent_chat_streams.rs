use crate::ActiveStreams;
use std::future::Future;
use std::sync::Arc;
use tokio_util::sync::CancellationToken;

pub(crate) type StreamEntry = (
    CancellationToken,
    u64,
    String,
    Arc<crate::services::agent_local::parent_message_inbox::ParentMessageInbox>,
);

const MAX_ACTIVE_STREAMS: usize = 32;

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
        if map.len() >= MAX_ACTIVE_STREAMS && !map.contains_key(session_id) {
            return Err("Trop de flux actifs simultanément".to_string());
        }
    }
    let request_id = start_request().await;
    let old_stream = {
        let mut map = streams.0.lock().await;
        if map.len() >= MAX_ACTIVE_STREAMS && !map.contains_key(session_id) {
            return Err("Trop de flux actifs simultanément".to_string());
        }
        let old_stream = map.insert(
            session_id.to_string(),
            (cancel.clone(), generation, request_id.clone(), inbox),
        );
        crate::services::agent_local::subagent_registry::adopt_children_for_parent_stream(
            session_id, &cancel,
        )
        .await;
        old_stream
    };
    if let Some(old_stream) = old_stream {
        cancel_previous(old_stream).await;
    }
    let is_current = matches!(
        streams.0.lock().await.get(session_id),
        Some((_, active_generation, _, _)) if *active_generation == generation
    );
    if !is_current {
        cancel.cancel();
        return Err("Requête remplacée par un flux plus récent".to_string());
    }
    Ok(request_id)
}
