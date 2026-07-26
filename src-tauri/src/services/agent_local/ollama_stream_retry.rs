use crate::services::agent_local::types_ollama::{ChatRequest, OllamaThink};

pub fn build_retry_request(request: &ChatRequest, error_body: &str) -> Option<ChatRequest> {
    let mut retry = request.clone();
    let mut changed = false;
    if error_body.contains("does not support thinking")
        && request.think.as_ref().is_some_and(|think| think.enabled())
    {
        retry.think = Some(OllamaThink::Bool(false));
        changed = true;
    }
    if error_body.contains("does not support tools") && request.tools.is_some() {
        retry.tools = None;
        changed = true;
    }
    if error_body.contains("does not support images") {
        for msg in &mut retry.messages {
            msg.images = None;
        }
        changed = true;
    }
    changed.then_some(retry)
}
