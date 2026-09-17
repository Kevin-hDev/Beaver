use super::OpenChatResponse;
use crate::services::agent_local::ollama_retry_indicator::server_retry_delay;
use tokio_util::sync::CancellationToken;

pub(in crate::services::agent_local) const CONNECTION: &str = "ollama_connection_lost";
pub(in crate::services::agent_local) const INVALID_RESPONSE: &str = "provider_response_invalid";
pub(in crate::services::agent_local) const SERVER: &str = "ollama_server_error";
pub(in crate::services::agent_local) const TIMEOUT: &str = "timeout";

pub(super) fn connection(error: reqwest::Error) -> Result<OpenChatResponse, String> {
    let code = if error.is_connect() || error.is_timeout() {
        CONNECTION
    } else {
        SERVER
    };
    Err(code.to_string())
}

pub(in crate::services::agent_local) fn invalid_response(error: &str) -> String {
    ::log::error!(
        "[ollama-stream] réponse invalide: {}",
        crate::services::llm::sanitize_log_body(error)
    );
    INVALID_RESPONSE.to_string()
}

pub(super) async fn wait_retry(cancel: &CancellationToken, attempt: u32) -> Result<(), String> {
    tokio::select! {
        _ = cancel.cancelled() => Err("Annulé".to_string()),
        _ = tokio::time::sleep(server_retry_delay(attempt)) => Ok(()),
    }
}
