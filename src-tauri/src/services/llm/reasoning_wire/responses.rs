use crate::services::reasoning_continuity::tool_links::ToolLink;
use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ResponseItemError {
    UnsupportedItem,
    InvalidFunctionCall,
}

/// Extrait seulement les items Responses qui sont réinjectables tels quels.
/// Un item final inconnu ferme la continuité plutôt que de risquer un rejeu incomplet.
pub(super) fn completed_item(event: &Value) -> Result<Option<Value>, ResponseItemError> {
    if event.get("type").and_then(Value::as_str) != Some("response.output_item.done") {
        return Ok(None);
    }
    event.get("item").map(validate_item).transpose()
}

/// Repli pour les réponses non streamées : l'ordre du tableau provider est conservé.
pub(super) fn final_items(event: &Value) -> Result<Vec<Value>, ResponseItemError> {
    if event.get("type").and_then(Value::as_str) != Some("response.completed") {
        return Ok(Vec::new());
    }
    event
        .pointer("/response/output")
        .and_then(Value::as_array)
        .map(|items| items.iter().map(validate_item).collect())
        .unwrap_or_else(|| Ok(Vec::new()))
}

pub(super) fn tool_link(item: &Value) -> Result<Option<ToolLink>, ResponseItemError> {
    if item.get("type").and_then(Value::as_str) != Some("function_call") {
        return Ok(None);
    }
    let provider_call_id = item
        .get("call_id")
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .ok_or(ResponseItemError::InvalidFunctionCall)?;
    let tool_name = item
        .get("name")
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .ok_or(ResponseItemError::InvalidFunctionCall)?;
    Ok(Some(ToolLink {
        provider_call_id: provider_call_id.to_owned(),
        tool_name: tool_name.to_owned(),
    }))
}

fn validate_item(item: &Value) -> Result<Value, ResponseItemError> {
    match item.get("type").and_then(Value::as_str) {
        Some("reasoning" | "message") => Ok(item.clone()),
        Some("function_call") => {
            tool_link(item)?;
            Ok(item.clone())
        }
        _ => Err(ResponseItemError::UnsupportedItem),
    }
}

#[cfg(test)]
#[path = "responses_tests.rs"]
mod tests;
