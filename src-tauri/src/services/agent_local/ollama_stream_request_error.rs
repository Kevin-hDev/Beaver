use super::OpenChatResponse;
use crate::services::agent_local::ollama_retry_indicator::server_retry_delay;
use crate::services::agent_local::stream_events::AgentEventEmitter;
use crate::services::agent_local::types_ollama::StreamEvent;
use tokio_util::sync::CancellationToken;

pub(super) fn connection(
    on_event: &AgentEventEmitter,
    error: reqwest::Error,
) -> Result<OpenChatResponse, String> {
    let is_connection = error.is_connect() || error.is_timeout();
    let msg = if is_connection {
        "ollama_connection_lost".to_string()
    } else {
        format!("Ollama: {error}")
    };
    let _ = on_event.send(StreamEvent::Error {
        message: msg.clone(),
        is_connection,
        context_capacity: None,
        diagnostic: None,
    });
    Err(msg)
}

pub(super) async fn wait_retry(cancel: &CancellationToken, attempt: u32) -> Result<(), String> {
    tokio::select! {
        _ = cancel.cancelled() => Err("Annulé".to_string()),
        _ = tokio::time::sleep(server_retry_delay(attempt)) => Ok(()),
    }
}
