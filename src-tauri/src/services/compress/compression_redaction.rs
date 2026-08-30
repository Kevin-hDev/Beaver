use crate::services::agent_local::types_message::AgentMessage;

pub fn redact_messages_for_compression(source: &[AgentMessage]) -> Vec<AgentMessage> {
    source
        .iter()
        .cloned()
        .map(|mut message| {
            redact_string(&mut message.content);
            redact_optional(&mut message.thinking);
            message.continuation = None;
            if let Some(calls) = &mut message.tool_calls {
                for call in calls {
                    crate::services::agent_local::sensitive_data::redact_json_preserving_shape(
                        &mut call.function.arguments,
                    );
                    if let Some(extra) = &mut call.extra_content {
                        crate::services::agent_local::sensitive_data::redact_json_preserving_shape(
                            extra,
                        );
                    }
                }
            }
            redact_serializable(&mut message.tool_activities);
            redact_serializable(&mut message.segments);
            for file in &mut message.files {
                redact_string(&mut file.name);
                redact_string(&mut file.path);
                redact_optional(&mut file.thumbnail);
                redact_optional(&mut file.access_grant);
            }
            message
        })
        .collect()
}

pub fn redact_checkpoint_text(source: &str) -> String {
    crate::services::agent_local::sensitive_data::redact_text(source)
}

fn redact_string(value: &mut String) {
    crate::services::agent_local::sensitive_data::redact_string(value);
}

fn redact_optional(value: &mut Option<String>) {
    if let Some(value) = value {
        redact_string(value);
    }
}

pub(super) fn redact_serializable<T>(value: &mut Option<Vec<T>>)
where
    T: serde::Serialize + serde::de::DeserializeOwned,
{
    let Some(items) = value.take() else {
        return;
    };
    let Ok(mut json) = serde_json::to_value(&items) else {
        return;
    };
    crate::services::agent_local::sensitive_data::redact_json_preserving_shape(&mut json);
    *value = serde_json::from_value(json).ok();
}
