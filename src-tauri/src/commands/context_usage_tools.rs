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

#[cfg(test)]
mod tests {
    use super::definition_tokens;
    use crate::services::agent_local::{
        context_usage_buckets::{ContextUsageSeed, RequestContextUsage},
        tool_definitions_extensions::list_extensions_definition_with_catalog,
    };

    fn assert_catalog(catalog: &str) {
        let definition = list_extensions_definition_with_catalog(catalog);
        let expected =
            crate::services::token_counting::estimate_text_tokens(&definition.to_string());
        assert_eq!(definition_tokens(vec![definition.clone()]), (expected, 0));
        let empty =
            RequestContextUsage::from_request("ollama", &[], &[], ContextUsageSeed::default());
        let usage = RequestContextUsage::from_request(
            "ollama",
            &[],
            &[definition],
            ContextUsageSeed::default(),
        );
        assert_eq!(
            usage.system_tools.saturating_sub(empty.system_tools),
            expected as u32
        );
    }
    #[test]
    fn empty_extension_catalog_definition_counts_its_serialized_value() {
        assert_catalog("");
    }
    #[test]
    fn normal_extension_catalog_definition_counts_its_serialized_value() {
        assert_catalog("[{\"name\":\"A\",\"id\":\"example.a\"}]");
    }
    #[test]
    fn maximal_extension_catalog_definition_counts_its_serialized_value() {
        let catalog = (0..132)
            .map(|index| {
                format!("{{\"name\":\"Extension {index}\",\"id\":\"example.extension{index}\"}}")
            })
            .collect::<Vec<_>>()
            .join(",");
        assert_catalog(&format!("[{catalog}]"));
    }
}
