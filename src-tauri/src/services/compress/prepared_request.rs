use crate::services::agent_local::context_usage_record::ContextTokenCount;
use crate::services::agent_local::types_ollama::ChatMessage;
use serde_json::{json, Value};

pub(crate) fn count(
    provider_id: &str,
    model: &str,
    messages: &[ChatMessage],
    tools: &[Value],
) -> ContextTokenCount {
    if crate::services::llm::route_profile::is_local(provider_id) {
        return ollama(model, messages, tools);
    }
    match crate::services::llm::route_profile::diagnostic_payload_kind(provider_id) {
        Some("responses") => responses(provider_id, model, messages, tools),
        Some("anthropic_messages") => {
            crate::services::llm::anthropic::prepared_context_count(messages, tools)
        }
        _ => chat(provider_id, model, messages, tools),
    }
}

fn ollama(model: &str, messages: &[ChatMessage], tools: &[Value]) -> ContextTokenCount {
    let Some(policy) = crate::services::llm::route_profile::payload_policy("ollama", model) else {
        return unknown();
    };
    let messages = crate::services::agent_local::ollama_tool_role::wrap_tool_results(
        messages,
        policy.message.tool_results,
    );
    crate::services::agent_local::prepared_context_count::ollama(&json!({
        "messages": crate::services::agent_local::ollama_wire::messages_value(&messages),
        "tools": tools,
    }))
}

pub(crate) fn system_head(
    provider_id: &str,
    model: &str,
    messages: &[ChatMessage],
    tools: &[Value],
) -> ContextTokenCount {
    let system = messages
        .iter()
        .filter(|message| matches!(message.role.as_str(), "system" | "developer"))
        .cloned()
        .collect::<Vec<_>>();
    count(provider_id, model, &system, tools)
}

fn chat(
    provider_id: &str,
    model: &str,
    messages: &[ChatMessage],
    tools: &[Value],
) -> ContextTokenCount {
    let Some(policy) = crate::services::llm::route_profile::payload_policy(provider_id, model)
        .map(|policy| policy.message)
    else {
        return unknown();
    };
    let messages = Value::Array(
        crate::services::llm::stream_convert::messages_to_openai_with_tools(
            messages, policy, tools,
        ),
    );
    crate::services::agent_local::prepared_context_count::chat_completions(&json!({
        "messages": messages,
        "tools": tools,
    }))
}

fn responses(
    provider_id: &str,
    model: &str,
    messages: &[ChatMessage],
    tools: &[Value],
) -> ContextTokenCount {
    let Some(payload_policy) =
        crate::services::llm::route_profile::payload_policy(provider_id, model)
    else {
        return unknown();
    };
    let Ok(converted) =
        crate::services::codex_client::convert::convert_messages_with_tools_and_continuity_evidence(
            messages,
            tools,
            None,
            payload_policy.message.tool_results,
        )
    else {
        return unknown();
    };
    let Some(tool_policy) = crate::services::llm::route_profile::tool_policy(provider_id, model)
    else {
        return unknown();
    };
    let native_tools =
        crate::services::codex_client::convert::convert_tools_to_responses_api(tool_policy, tools);
    crate::services::agent_local::prepared_context_count::responses(&json!({
        "instructions": converted.instructions,
        "input": converted.input,
        "tools": native_tools,
    }))
}

fn unknown() -> ContextTokenCount {
    ContextTokenCount {
        tokens: None,
        capacity_tokens: None,
        source: None,
        coverage: crate::services::agent_local::context_usage_record::ContextCountCoverage::Unknown,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn newly_discovered_tool_changes_the_next_candidate_count() {
        let messages = [ChatMessage::user("continue".into())];
        let before = count("openai", "gpt-5.6-luna", &messages, &[])
            .capacity_tokens
            .unwrap();
        let tools = [json!({
            "type": "function",
            "function": {"name": "new_tool", "description": "discovered", "parameters": {"type": "object"}}
        })];
        let after = count("openai", "gpt-5.6-luna", &messages, &tools)
            .capacity_tokens
            .unwrap();

        assert!(after > before);
    }

    #[test]
    fn ollama_tool_results_use_the_native_wire_shape() {
        let messages = [ChatMessage::tool(
            "result".into(),
            Some("call-1".into()),
            Some("read_file".into()),
        )];

        let count = count("ollama", "fixture", &messages, &[]);

        assert!(count.capacity_tokens.is_some());
    }
}
