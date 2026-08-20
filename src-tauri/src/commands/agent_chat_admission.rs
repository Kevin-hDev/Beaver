use super::agent_chat_streams::StreamEntry;
use crate::ActiveStreams;
use std::future::Future;
use std::sync::Arc;
use tokio_util::sync::CancellationToken;

pub(crate) struct AgentChatAdmission {
    pub cancel: CancellationToken,
    pub generation: u64,
    pub parent_message_inbox:
        Arc<crate::services::agent_local::parent_message_inbox::ParentMessageInbox>,
    pub permission_mode: String,
    pub request_id: String,
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
