use super::ReasoningCapture;
use crate::services::reasoning_continuity::{envelope::ContinuationState, limits::LimitError};
use serde_json::Value;

impl ReasoningCapture {
    pub(super) fn append_openrouter(&mut self, event: &Value) -> Result<(), LimitError> {
        let delta = event.pointer("/choices/0/delta/reasoning_details");
        let Some(items) = delta
            .or_else(|| event.pointer("/choices/0/message/reasoning_details"))
            .and_then(Value::as_array)
        else {
            return Ok(());
        };
        for item in items {
            let Some(ContinuationState::OpenRouterDetails { details }) = self.continuation.as_mut()
            else {
                return Err(LimitError::CaptureSkeleton);
            };
            let merge = delta
                .is_some()
                .then(|| details.last_mut())
                .flatten()
                .and_then(|last| merge_field(last, item).map(|field| (last, field)));
            if let Some((last, field)) = merge {
                self.budget
                    .as_mut()
                    .ok_or(LimitError::CaptureSkeleton)?
                    .observe_fragment(item)?;
                merge_delta(last, item, field)?;
            } else {
                self.append_items(vec![item.clone()])?;
            }
        }
        Ok(())
    }
}

// 2026-09-07: the official OpenRouter SDK joins consecutive text/summary deltas,
// never encrypted blobs. Preserve boundaries when metadata conflicts, rather
// than overwriting an identity/signature. Full non-stream messages stay intact.
// https://github.com/OpenRouterTeam/ai-sdk-provider/blob/main/src/chat/index.ts
fn merge_field(last: &Value, next: &Value) -> Option<&'static str> {
    let field = match next.get("type")?.as_str()? {
        "reasoning.text" => "text",
        "reasoning.summary" => "summary",
        _ => return None,
    };
    if last.get("type") != next.get("type") {
        return None;
    }
    for item in [last, next] {
        // A future dual-text shape is opaque until its merge contract is known.
        if item
            .get(if field == "text" { "summary" } else { "text" })
            .is_some()
        {
            return None;
        }
        if item.as_object()?.keys().any(|key| {
            !matches!(
                key.as_str(),
                "type" | "text" | "summary" | "index" | "id" | "format" | "signature"
            )
        }) || item
            .get(field)
            .is_some_and(|value| !value.is_null() && !value.is_string())
        {
            return None;
        }
    }
    for key in ["id", "index", "format"] {
        if let (Some(a), Some(b)) = (last.get(key), next.get(key)) {
            if !a.is_null() && !b.is_null() && a != b {
                return None;
            }
        }
    }
    // Signatures are opaque: do not compare, concatenate, or discard either.
    if last.get("signature").is_some_and(|v| !v.is_null())
        && next.get("signature").is_some_and(|v| !v.is_null())
    {
        return None;
    }
    Some(field)
}

fn merge_delta(last: &mut Value, next: &Value, field: &str) -> Result<(), LimitError> {
    let object = last.as_object_mut().ok_or(LimitError::CaptureSkeleton)?;
    if let Some(fragment) = next.get(field).and_then(Value::as_str) {
        let entry = object
            .entry(field)
            .or_insert_with(|| Value::String(String::new()));
        if entry.is_null() {
            *entry = Value::String(String::new());
        }
        let Value::String(text) = entry else {
            return Err(LimitError::CaptureSkeleton);
        };
        text.push_str(fragment);
    }
    for (key, value) in next.as_object().ok_or(LimitError::CaptureSkeleton)? {
        if key != field && object.get(key).is_none_or(Value::is_null) {
            object.insert(key.clone(), value.clone());
        }
    }
    Ok(())
}
