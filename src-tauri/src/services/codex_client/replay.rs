use zeroize::Zeroizing;

use crate::services::agent_local::types_ollama::{ChatMessage, StreamResult};

const MAX_REPLAY_ITEMS: usize = 128;
const MAX_REPLAY_BYTES: usize = 8 * 1024 * 1024;
const CODEX_ITEMS_PATH: &str = "/codex/output_items";

#[derive(Default)]
pub struct ReplayCollector {
    items: Vec<serde_json::Value>,
    bytes: usize,
}

impl ReplayCollector {
    pub fn capture(&mut self, item: &serde_json::Value) -> Result<(), String> {
        if !is_replayable(item) {
            return Ok(());
        }
        if self.items.len() >= MAX_REPLAY_ITEMS {
            return Err("codex_response_state_too_large".to_string());
        }
        let item_bytes =
            json_size(item).ok_or_else(|| "codex_response_state_invalid".to_string())?;
        let next = self
            .bytes
            .checked_add(item_bytes)
            .ok_or_else(|| "codex_response_state_too_large".to_string())?;
        if next > MAX_REPLAY_BYTES {
            return Err("codex_response_state_too_large".to_string());
        }
        self.bytes = next;
        self.items.push(item.clone());
        Ok(())
    }

    pub fn attach(self, result: &mut StreamResult) {
        if result.tool_calls.is_empty() || self.items.is_empty() {
            return;
        }
        result
            .tool_call_extra_content
            .resize(result.tool_calls.len(), None);
        result.tool_call_extra_content[0] = Some(serde_json::json!({
            "codex": { "output_items": self.items }
        }));
    }
}

pub fn restore_tool_name(item: &mut serde_json::Value, tools: &[serde_json::Value]) {
    if item.get("type").and_then(serde_json::Value::as_str) != Some("function_call") {
        return;
    }
    let Some(name) = item.get("name").and_then(serde_json::Value::as_str) else {
        return;
    };
    item["name"] = crate::services::llm::tool_schema::restore_tool_name(name, tools).into();
}

pub fn items_from_message(message: &ChatMessage) -> Option<Vec<serde_json::Value>> {
    let calls = message.tool_calls.as_ref()?;
    let items = calls
        .iter()
        .find_map(|call| call.extra_content.as_ref()?.pointer(CODEX_ITEMS_PATH))?
        .as_array()?;
    if items.is_empty() || items.len() > MAX_REPLAY_ITEMS {
        return None;
    }
    let mut bytes = 0usize;
    for item in items {
        if !is_replayable(item) {
            return None;
        }
        bytes = bytes.checked_add(json_size(item)?)?;
        if bytes > MAX_REPLAY_BYTES {
            return None;
        }
    }
    if !function_calls_match(calls, items) {
        return None;
    }
    Some(items.clone())
}

fn function_calls_match(
    calls: &[crate::services::agent_local::types_ollama::ToolCallOllama],
    items: &[serde_json::Value],
) -> bool {
    let replayed: Vec<_> = items
        .iter()
        .filter(|item| {
            item.get("type").and_then(serde_json::Value::as_str) == Some("function_call")
        })
        .collect();
    replayed.len() == calls.len()
        && replayed.iter().zip(calls).all(|(item, call)| {
            item.get("call_id").and_then(serde_json::Value::as_str) == call.id.as_deref()
                && item.get("name").and_then(serde_json::Value::as_str)
                    == Some(call.function.name.as_str())
        })
}

fn is_replayable(item: &serde_json::Value) -> bool {
    matches!(
        item.get("type").and_then(serde_json::Value::as_str),
        Some("reasoning" | "message" | "function_call")
    )
}

fn json_size(value: &serde_json::Value) -> Option<usize> {
    serde_json::to_vec(value)
        .ok()
        .map(Zeroizing::new)
        .map(|serialized| serialized.len())
}

#[cfg(test)]
#[path = "replay_tests.rs"]
mod tests;
