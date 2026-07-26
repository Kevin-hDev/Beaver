use crate::services::agent_local::types_ollama::ChatMessage;
use crate::services::llm::vision;
use serde_json::{json, Value};

pub fn message_to_openai(msg: &ChatMessage, provider_id: &str) -> Value {
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
            let content = if msg.content.is_empty() && msg.tool_calls.is_some() {
                Value::Null
            } else {
                json!(msg.content)
            };
            let mut obj = json!({
                "role": "assistant",
                "content": content,
            });
            if let Some(rc) = &msg.reasoning_content {
                obj["reasoning_content"] = json!(rc);
            }
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
                                "name": tc.function.name,
                                "arguments": args_str,
                            }
                        })
                    })
                    .collect();
                for (value, tc) in tc_arr.iter_mut().zip(tcs.iter()) {
                    if let Some(extra_content) = &tc.extra_content {
                        value["extra_content"] = extra_content.clone();
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

pub fn messages_to_openai(messages: &[ChatMessage], provider_id: &str) -> Vec<Value> {
    messages
        .iter()
        .map(|m| message_to_openai(m, provider_id))
        .collect()
}

#[cfg(test)]
#[path = "stream_convert_tests.rs"]
mod tests;
