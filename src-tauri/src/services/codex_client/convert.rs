use super::replay;
use crate::services::agent_local::types_ollama::ChatMessage;
use crate::services::llm::vision;

pub fn convert_messages(messages: &[ChatMessage]) -> (String, Vec<serde_json::Value>) {
    convert_messages_with_tools(messages, &[])
}

pub fn convert_messages_with_tools(
    messages: &[ChatMessage],
    tools: &[serde_json::Value],
) -> (String, Vec<serde_json::Value>) {
    convert_messages_with_tools_and_continuity(messages, tools, None)
        .expect("a payload without a continuation target cannot be rejected")
}

pub(crate) fn convert_messages_with_tools_and_continuity(
    messages: &[ChatMessage],
    tools: &[serde_json::Value],
    target: Option<&crate::services::reasoning_continuity::contract::ContinuationTarget>,
) -> Result<
    (String, Vec<serde_json::Value>),
    crate::services::llm::reasoning_wire::replay::ReplayApplyError,
> {
    let target =
        crate::services::llm::reasoning_wire::responses::target_for_request(messages, target)?;
    let mut instructions = String::new();
    let mut input = Vec::new();
    let tool_names = crate::services::llm::tool_schema::ToolNameMap::new(tools);

    for msg in messages {
        if msg.role == "system" {
            if !instructions.is_empty() {
                instructions.push_str("\n\n");
            }
            instructions.push_str(&msg.content);
            continue;
        }

        if msg.role == "assistant" {
            if let Some(items) = crate::services::llm::reasoning_wire::responses::items_for_message(
                msg,
                target.as_ref(),
            )? {
                input.extend(items);
                continue;
            }
            if let Some(mut items) = replay::items_from_message(msg) {
                alias_replay_tool_names(&mut items, &tool_names);
                input.extend(items);
                continue;
            }
            if !msg.content.is_empty() {
                input.push(serde_json::json!({"role": "assistant", "content": msg.content}));
            }
            if let Some(ref calls) = msg.tool_calls {
                for tc in calls {
                    let args = match &tc.function.arguments {
                        serde_json::Value::String(s) => s.clone(),
                        other => serde_json::to_string(other).unwrap_or_default(),
                    };
                    input.push(serde_json::json!({
                        "type": "function_call",
                        "call_id": tc.id.as_deref().unwrap_or("call_0"),
                        "name": tool_names.wire_name(&tc.function.name),
                        "arguments": args,
                    }));
                }
            }
            continue;
        }

        if msg.role == "tool" {
            if let Some(ref id) = msg.tool_call_id {
                input.push(serde_json::json!({
                    "type": "function_call_output",
                    "call_id": id,
                    "output": msg.content,
                }));
                continue;
            }
        }

        if msg.role == "user" {
            input.push(user_message_to_responses(msg));
        } else {
            input.push(serde_json::json!({"role": msg.role, "content": msg.content}));
        }
    }
    Ok((instructions, input))
}

fn alias_replay_tool_names(
    items: &mut [serde_json::Value],
    tool_names: &crate::services::llm::tool_schema::ToolNameMap,
) {
    for item in items {
        if item.get("type").and_then(serde_json::Value::as_str) != Some("function_call") {
            continue;
        }
        let Some(name) = item.get("name").and_then(serde_json::Value::as_str) else {
            continue;
        };
        item["name"] = tool_names.wire_name(name).into();
    }
}

/// Les transports Responses (Codex, OpenAI API et xAI OAuth) réutiliseront
/// cette conversion unique lorsqu'une politique sera validée réel.
#[allow(
    dead_code,
    reason = "Task 19 connects this only after a live-validated Responses policy"
)]
pub(crate) fn convert_continuity(
    messages: &[ChatMessage],
    approval: &crate::services::llm::reasoning_wire::replay::ReplayApproval<'_>,
    input: &mut Vec<serde_json::Value>,
) -> Result<(), crate::services::llm::reasoning_wire::replay::ReplayApplyError> {
    crate::services::llm::reasoning_wire::replay::apply_responses_continuity(
        messages, approval, input,
    )
}

fn user_message_to_responses(msg: &ChatMessage) -> serde_json::Value {
    let Some(images) = &msg.images else {
        return serde_json::json!({"role": "user", "content": msg.content});
    };
    if images.is_empty() {
        return serde_json::json!({"role": "user", "content": msg.content});
    }

    let mut content = Vec::new();
    if !msg.content.is_empty() {
        content.push(serde_json::json!({"type": "input_text", "text": msg.content}));
    }
    for image in images {
        content.push(vision::responses_image_part(image));
    }
    serde_json::json!({"role": "user", "content": content})
}

fn fix_array_schemas(v: &mut serde_json::Value) {
    match v {
        serde_json::Value::Object(map) => {
            if map.get("type").and_then(|t| t.as_str()) == Some("array")
                && !map.contains_key("items")
            {
                map.insert("items".to_string(), serde_json::json!({"type": "string"}));
            }
            for val in map.values_mut() {
                fix_array_schemas(val);
            }
        }
        serde_json::Value::Array(arr) => {
            for val in arr {
                fix_array_schemas(val);
            }
        }
        _ => {}
    }
}

pub fn convert_tools_to_responses_api(
    provider_id: &str,
    model: &str,
    tools: &[serde_json::Value],
) -> Vec<serde_json::Value> {
    crate::services::llm::tool_schema::tools_for_provider(provider_id, model, tools)
        .iter()
        .filter_map(|t| {
            let func = t.get("function")?;
            let mut params = func
                .get("parameters")
                .cloned()
                .unwrap_or(serde_json::Value::Null);
            fix_array_schemas(&mut params);
            Some(serde_json::json!({
                "type": "function",
                "name": func.get("name")?,
                "description": func.get("description").unwrap_or(&serde_json::Value::Null),
                "parameters": params,
                "strict": func.get("strict").unwrap_or(&serde_json::Value::Bool(false)),
            }))
        })
        .collect()
}

#[cfg(test)]
#[path = "convert_tests.rs"]
mod tests;
