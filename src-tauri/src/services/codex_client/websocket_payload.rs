use super::{CodexRequest, WebSocketFailure};
use crate::services::secure_http::LLM_BODY_LIMIT;

pub(super) fn build_payload(request: &CodexRequest) -> Result<String, WebSocketFailure> {
    let mut payload = serde_json::to_value(request)
        .map_err(|_| WebSocketFailure::Unavailable { partial: false })?;
    let object = payload
        .as_object_mut()
        .ok_or(WebSocketFailure::Unavailable { partial: false })?;
    object.insert("type".to_string(), "response.create".into());
    let payload = serde_json::to_string(&payload)
        .map_err(|_| WebSocketFailure::Unavailable { partial: false })?;
    if payload.len() > LLM_BODY_LIMIT {
        return Err(WebSocketFailure::Unavailable { partial: false });
    }
    Ok(payload)
}
