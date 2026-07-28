use std::collections::HashSet;

use serde_json::Value;

use super::extension_tool_selection::{CapacityDecision, PluginDescriptor};

pub fn plugin_descriptors(tools: &[Value]) -> Vec<PluginDescriptor> {
    crate::services::extensions::indexed_plugins()
        .into_iter()
        .map(|plugin| {
            let names = plugin
                .tools
                .iter()
                .map(|tool| tool.name.as_str())
                .collect::<HashSet<_>>();
            PluginDescriptor {
                id: plugin.id,
                tool_count: tools
                    .iter()
                    .filter_map(definition_name)
                    .filter(|name| names.contains(name))
                    .count(),
                replaces_core: plugin.tools.iter().any(|tool| tool.replaces_core),
            }
        })
        .collect()
}

pub fn active_definitions(
    tools: &[Value],
    decision: &CapacityDecision,
    provider_tool_limit: usize,
) -> Vec<Value> {
    active_definitions_with(tools, decision, provider_tool_limit, |name| {
        crate::services::extensions::plugin_id_for_tool(name)
    })
}

pub fn active_definitions_with(
    tools: &[Value],
    decision: &CapacityDecision,
    provider_tool_limit: usize,
    plugin_id_for_tool: impl Fn(&str) -> Option<String>,
) -> Vec<Value> {
    let active = decision
        .active_plugin_ids
        .iter()
        .map(String::as_str)
        .collect::<HashSet<_>>();
    let mut selected = tools
        .iter()
        .filter(|tool| {
            definition_name(tool)
                .and_then(&plugin_id_for_tool)
                .is_none_or(|plugin_id| active.contains(plugin_id.as_str()))
        })
        .cloned()
        .collect::<Vec<_>>();
    if selected.len() <= provider_tool_limit {
        return selected;
    }
    let discovery = selected
        .iter()
        .find(|tool| {
            definition_name(tool) == Some(crate::services::extensions::SEARCH_TOOL_NAME)
        })
        .cloned();
    selected.truncate(provider_tool_limit);
    if provider_tool_limit > 0
        && !selected.iter().any(|tool| {
            definition_name(tool) == Some(crate::services::extensions::SEARCH_TOOL_NAME)
        })
    {
        if let Some(discovery) = discovery {
            selected[provider_tool_limit - 1] = discovery;
        }
    }
    selected
}

pub fn append_capacity_notice(tools: &mut [Value], omitted: &[String]) {
    if omitted.is_empty() {
        return;
    }
    let Some(description) = tools.iter_mut().find_map(|tool| {
        (definition_name(tool) == Some(crate::services::extensions::SEARCH_TOOL_NAME))
            .then(|| tool.pointer_mut("/function/description"))
            .flatten()
            .and_then(|value| value.as_str())
            .map(str::to_string)
    }) else {
        return;
    };
    let shown = omitted
        .iter()
        .take(super::provider_tool_limits::MAX_CAPACITY_DIAGNOSTIC_ITEMS)
        .cloned()
        .collect::<Vec<_>>()
        .join(", ");
    let remaining = omitted
        .len()
        .saturating_sub(super::provider_tool_limits::MAX_CAPACITY_DIAGNOSTIC_ITEMS);
    let suffix = if remaining > 0 {
        format!(" (+{remaining} additional plugins)")
    } else {
        String::new()
    };
    let notice = format!(
        "\n\nProvider limit: tools from {shown}{suffix} are not loaded in this request. \
         Use search_extension_tools to load a needed plugin."
    );
    if let Some(tool) = tools.iter_mut().find(|tool| {
        definition_name(tool) == Some(crate::services::extensions::SEARCH_TOOL_NAME)
    }) {
        tool["function"]["description"] = Value::String(format!("{description}{notice}"));
    }
}

pub fn definition_name(definition: &Value) -> Option<&str> {
    definition.pointer("/function/name").and_then(Value::as_str)
}
