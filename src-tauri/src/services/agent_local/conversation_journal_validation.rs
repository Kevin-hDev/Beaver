use super::ChatMessage;
use std::collections::HashSet;

pub(crate) fn validate_tool_results(
    messages: &[ChatMessage],
    expected: &[String],
) -> Result<(), String> {
    if messages.iter().any(|message| message.role != "tool") {
        return Err(error());
    }
    let actual = messages
        .iter()
        .map(|message| message.tool_call_id.clone().ok_or_else(error))
        .collect::<Result<Vec<_>, _>>()?;
    if actual != expected || actual.iter().collect::<HashSet<_>>().len() != actual.len() {
        return Err(error());
    }
    Ok(())
}

pub(super) fn assistant_tool_ids(message: &ChatMessage) -> Result<Vec<String>, String> {
    let ids = message
        .tool_calls
        .as_deref()
        .unwrap_or_default()
        .iter()
        .map(|call| call.id.clone().ok_or_else(error))
        .collect::<Result<Vec<_>, _>>()?;
    (ids.iter().collect::<HashSet<_>>().len() == ids.len())
        .then_some(ids)
        .ok_or_else(error)
}

pub(super) fn error() -> String {
    "conversation_journal_failed".to_string()
}
