use crate::services::agent_local::types_ollama::ChatMessage;
use crate::services::llm::vision;
use serde_json::{json, Value};

#[cfg(test)]
pub fn message_to_openai(msg: &ChatMessage, provider_id: &str) -> Value {
    let names = super::tool_schema::ToolNameMap::new(&[]);
    message_to_openai_with_names(msg, provider_id, &names)
}

fn message_to_openai_with_names(
    msg: &ChatMessage,
    provider_id: &str,
    names: &super::tool_schema::ToolNameMap,
) -> Value {
    match msg.role.as_str() {
        "tool" => {
            let mut obj = json!({
                "role": "tool",
                "content": msg.content,
            });
            if let Some(id) = &msg.tool_call_id {
                obj["tool_call_id"] = json!(id);
            }
            obj
        }
        "assistant" => {
            let content = if msg.content.is_empty()
                && msg.tool_calls.is_some()
                && provider_id != "deepseek"
            {
                Value::Null
            } else {
                json!(msg.content)
            };
            let mut obj = json!({
                "role": "assistant",
                "content": content,
            });
            if let Some(tcs) = &msg.tool_calls {
                let mut tc_arr: Vec<Value> = tcs
                    .iter()
                    .enumerate()
                    .map(|(i, tc)| {
                        let args_str = serde_json::to_string(&tc.function.arguments)
                            .unwrap_or_else(|_| "{}".to_string());
                        let id = tc.id.clone().unwrap_or_else(|| format!("call_{}", i));
                        json!({
                            "id": id,
                            "type": "function",
                            "function": {
                                "name": names.wire_name(&tc.function.name),
                                "arguments": args_str,
                            }
                        })
                    })
                    .collect();
                for (value, tc) in tc_arr.iter_mut().zip(tcs.iter()) {
                    if let Some(extra_content) = &tc.extra_content {
                        if let Some(extra_content) =
                            extra_content_for_provider(extra_content, provider_id)
                        {
                            value["extra_content"] = extra_content;
                        }
                    }
                }
                obj["tool_calls"] = json!(tc_arr);
            }
            obj
        }
        "user" => {
            if let Some(images) = &msg.images {
                if !images.is_empty() {
                    let mut parts = vec![json!({"type": "text", "text": msg.content})];
                    for img in images {
                        parts.push(vision::openai_image_part(img, provider_id));
                    }
                    return json!({ "role": "user", "content": parts });
                }
            }
            json!({ "role": "user", "content": msg.content })
        }
        _ => {
            json!({ "role": msg.role, "content": msg.content })
        }
    }
}

fn extra_content_for_provider(extra: &Value, provider_id: &str) -> Option<Value> {
    if provider_id == crate::services::codex_client::PROVIDER_ID {
        return Some(extra.clone());
    }
    let mut filtered = extra.clone();
    if let Some(object) = filtered.as_object_mut() {
        object.remove("codex");
        if object.is_empty() {
            return None;
        }
    }
    Some(filtered)
}

pub fn messages_to_openai(messages: &[ChatMessage], provider_id: &str) -> Vec<Value> {
    messages_to_openai_with_tools(messages, provider_id, &[])
}

pub fn messages_to_openai_with_tools(
    messages: &[ChatMessage],
    provider_id: &str,
    tools: &[Value],
) -> Vec<Value> {
    let names = super::tool_schema::ToolNameMap::new(tools);
    messages
        .iter()
        .map(|message| message_to_openai_with_names(message, provider_id, &names))
        .collect()
}

#[cfg(test)]
#[path = "stream_convert_tests.rs"]
mod tests;
