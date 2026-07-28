use crate::services::agent_local::{
    tool_catalog, tool_definitions_chat, tool_definitions_mcp, tool_dispatcher,
};
use serde_json::Value;

pub(super) fn filtered_definitions(
    mode: &str,
    has_tools: bool,
    enabled_optional_tools: &[String],
) -> Vec<Value> {
    if !has_tools {
        return vec![];
    }
    let definitions = if mode == "chat" {
        tool_definitions_chat::get_chat_tool_definitions()
    } else {
        tool_dispatcher::get_tool_definitions()
    };
    tool_catalog::filter_tool_definitions(definitions, enabled_optional_tools)
}

pub(super) fn definition_tokens(definitions: Vec<Value>) -> (usize, usize) {
    let mcp_names = mcp_tool_names();
    definitions
        .into_iter()
        .fold((0, 0), |(system, mcp), definition| {
            let tokens = estimate(&definition.to_string());
            if tool_name(&definition).is_some_and(|name| mcp_names.contains(&name)) {
                (system, mcp + tokens)
            } else {
                (system + tokens, mcp)
            }
        })
}

fn mcp_tool_names() -> Vec<String> {
    tool_definitions_mcp::mcp_tool_definitions()
        .iter()
        .filter_map(tool_name)
        .collect()
}

fn tool_name(definition: &Value) -> Option<String> {
    definition
        .pointer("/function/name")
        .and_then(Value::as_str)
        .map(ToString::to_string)
}

fn estimate(input: &str) -> usize {
    crate::services::token_counting::estimate_text_tokens(input)
}
