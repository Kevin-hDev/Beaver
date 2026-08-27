use super::types_ollama::{ChatMessage, ChatRequest};
use serde_json::{json, Map, Value};

pub fn chat_request(
    request: &ChatRequest,
    messages: &[ChatMessage],
) -> Result<Value, crate::services::llm::reasoning_wire::replay::ReplayApplyError> {
    chat_request_with_evidence(request, messages).map(|prepared| prepared.payload)
}

pub(crate) struct PreparedOllamaPayload {
    pub payload: Value,
    pub replayed: Vec<crate::services::llm::reasoning_wire::replay::ReplayEvidence>,
}

pub(crate) fn chat_request_with_evidence(
    request: &ChatRequest,
    messages: &[ChatMessage],
) -> Result<PreparedOllamaPayload, crate::services::llm::reasoning_wire::replay::ReplayApplyError> {
    let mut payload_messages = messages_value(messages);
    let mut replayed = Vec::new();
    if let Some(target) = request.live_replay_target.as_ref() {
        replayed.extend(apply_live_continuity(
            messages,
            target,
            payload_messages.as_array_mut().ok_or(
                crate::services::llm::reasoning_wire::replay::ReplayApplyError::PayloadMismatch,
            )?,
        )?);
    }
    #[cfg(debug_assertions)]
    if let Some(target) = request.fixture_candidate.as_ref() {
        replayed.extend(apply_fixture_continuity(
            messages,
            target,
            payload_messages.as_array_mut().ok_or(
                crate::services::llm::reasoning_wire::replay::ReplayApplyError::PayloadMismatch,
            )?,
        )?);
    }
    let mut body = Map::new();
    body.insert("model".into(), json!(request.model));
    body.insert("messages".into(), payload_messages);
    body.insert("stream".into(), json!(request.stream));
    body.insert("truncate".into(), json!(false));
    insert_optional(&mut body, "tools", request.tools.as_ref());
    insert_optional(&mut body, "options", request.options.as_ref());
    insert_optional(&mut body, "keep_alive", request.keep_alive.as_ref());
    insert_optional(&mut body, "think", request.think.as_ref());
    Ok(PreparedOllamaPayload {
        payload: Value::Object(body),
        replayed,
    })
}

pub fn messages_value(messages: &[ChatMessage]) -> Value {
    Value::Array(messages.iter().map(message_value).collect())
}

/// Réservé au contrat natif `/api/chat` et appelé seulement après l'autorisation
/// exacte du registre.
pub(crate) fn apply_continuity(
    messages: &[ChatMessage],
    approval: &crate::services::llm::reasoning_wire::replay::ReplayApproval<'_>,
    payload_messages: &mut [Value],
) -> Result<(), crate::services::llm::reasoning_wire::replay::ReplayApplyError> {
    crate::services::llm::reasoning_wire::replay::apply_ollama_continuity(
        messages,
        approval,
        payload_messages,
    )
}

fn apply_live_continuity(
    messages: &[ChatMessage],
    target: &crate::services::reasoning_continuity::contract::ReplayTarget,
    payload_messages: &mut [Value],
) -> Result<
    Vec<crate::services::llm::reasoning_wire::replay::ReplayEvidence>,
    crate::services::llm::reasoning_wire::replay::ReplayApplyError,
> {
    let continuation =
        crate::services::reasoning_continuity::contract::ContinuationTarget::Replay(target.clone());
    let mut replayed = Vec::new();
    for message in messages
        .iter()
        .filter(|message| message.continuation.is_some())
    {
        let envelope = message
            .continuation
            .as_ref()
            .expect("filtered continuation");
        let approval = crate::services::llm::reasoning_wire::replay::approval_for_target(
            &continuation,
            envelope,
        )?;
        apply_continuity(messages, &approval, payload_messages)?;
        replayed.push(
            crate::services::llm::reasoning_wire::replay::ReplayEvidence::from_message(message)?,
        );
    }
    Ok(replayed)
}

#[cfg(debug_assertions)]
fn apply_fixture_continuity(
    messages: &[ChatMessage],
    target: &crate::services::reasoning_continuity::contract::ReplayTarget,
    payload_messages: &mut [Value],
) -> Result<
    Vec<crate::services::llm::reasoning_wire::replay::ReplayEvidence>,
    crate::services::llm::reasoning_wire::replay::ReplayApplyError,
> {
    let continuation =
        crate::services::reasoning_continuity::contract::ContinuationTarget::FixtureCandidate(
            target.clone(),
        );
    let mut replayed = Vec::new();
    for message in messages
        .iter()
        .filter(|message| message.continuation.is_some())
    {
        let envelope = message
            .continuation
            .as_ref()
            .expect("filtered continuation");
        let approval = crate::services::llm::reasoning_wire::replay::approval_for_target(
            &continuation,
            envelope,
        )?;
        apply_continuity(messages, &approval, payload_messages)?;
        replayed.push(
            crate::services::llm::reasoning_wire::replay::ReplayEvidence::from_message(message)?,
        );
    }
    Ok(replayed)
}

fn message_value(message: &ChatMessage) -> Value {
    let mut value = Map::new();
    value.insert("role".into(), json!(message.role));
    value.insert("content".into(), json!(message.content));
    insert_optional(&mut value, "images", message.images.as_ref());
    insert_tool_calls(&mut value, message);
    insert_optional(&mut value, "tool_name", message.tool_name.as_ref());
    insert_optional(&mut value, "thinking", message.tool_loop_reasoning.as_ref());
    Value::Object(value)
}

fn insert_tool_calls(value: &mut Map<String, Value>, message: &ChatMessage) {
    let Some(calls) = message.tool_calls.as_ref() else {
        return;
    };
    let calls = calls
        .iter()
        .map(|call| json!({ "function": call.function }))
        .collect();
    value.insert("tool_calls".into(), Value::Array(calls));
}

fn insert_optional<T: serde::Serialize>(
    value: &mut Map<String, Value>,
    key: &str,
    item: Option<&T>,
) {
    if let Some(item) = item {
        value.insert(key.into(), json!(item));
    }
}

#[cfg(test)]
#[path = "ollama_wire_tests.rs"]
mod tests;
