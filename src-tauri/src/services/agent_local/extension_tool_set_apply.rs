use std::collections::HashSet;

use serde_json::Value;

use super::extension_tool_selection::{CapacityDecision, PluginDescriptor};

pub struct ActiveDefinitions {
    pub tools: Vec<Value>,
    pub omitted_tool_names: Vec<String>,
    pub additional_omitted_tools: usize,
}

pub fn plugin_descriptors(tools: &[Value]) -> Vec<PluginDescriptor> {
    crate::services::extensions::indexed_plugins()
        .into_iter()
        .filter_map(|plugin| {
            let names = plugin
                .tools
                .iter()
                .map(|tool| tool.name.as_str())
                .collect::<HashSet<_>>();
            let definitions = tools
                .iter()
                .filter(|tool| {
                    definition_name(tool).is_some_and(|name| names.contains(name))
                })
                .collect::<Vec<_>>();
            if definitions.is_empty() && !plugin.tools.is_empty() {
                return None;
            }
            Some(PluginDescriptor {
                id: plugin.id,
                tool_count: definitions
                    .iter()
                    .filter(|tool| {
                        crate::services::extensions::core_fallback(tool).is_none()
                    })
                    .count(),
                definition_count: definitions.len(),
            })
        })
        .collect()
}

pub fn base_tool_count(tools: &[Value]) -> usize {
    tools
        .iter()
        .filter(|tool| {
            definition_name(tool)
                .is_none_or(|name| !crate::services::extensions::is_dynamic_tool(name))
                || crate::services::extensions::core_fallback(tool).is_some()
        })
        .count()
}

pub fn active_definitions(
    tools: &[Value],
    decision: &CapacityDecision,
    provider_tool_limit: usize,
) -> ActiveDefinitions {
    active_definitions_with(tools, decision, provider_tool_limit, |name| {
        crate::services::extensions::plugin_id_for_tool(name)
    })
}

pub fn active_definitions_with(
    tools: &[Value],
    decision: &CapacityDecision,
    provider_tool_limit: usize,
    plugin_id_for_tool: impl Fn(&str) -> Option<String>,
) -> ActiveDefinitions {
    let active = decision
        .active_plugin_ids
        .iter()
        .map(String::as_str)
        .collect::<HashSet<_>>();
    let selected = tools
        .iter()
        .filter_map(|tool| {
            let plugin_id = definition_name(tool).and_then(&plugin_id_for_tool);
            match plugin_id {
                Some(plugin_id) if active.contains(plugin_id.as_str()) => {
                    Some(crate::services::extensions::without_core_fallback(tool.clone()))
                }
                Some(_) => crate::services::extensions::core_fallback(tool).cloned(),
                None => Some(crate::services::extensions::without_core_fallback(tool.clone())),
            }
        })
        .collect::<Vec<_>>();
    cap_definitions(selected, provider_tool_limit)
}

fn cap_definitions(tools: Vec<Value>, provider_tool_limit: usize) -> ActiveDefinitions {
    if tools.len() <= provider_tool_limit {
        return ActiveDefinitions {
            tools,
            omitted_tool_names: Vec::new(),
            additional_omitted_tools: 0,
        };
    }
    let discovery_index = tools
        .iter()
        .position(|tool| {
            definition_name(tool) == Some(crate::services::extensions::SEARCH_TOOL_NAME)
        });
    let mut kept_indices = (0..provider_tool_limit).collect::<Vec<_>>();
    if let Some(index) = discovery_index.filter(|index| *index >= provider_tool_limit) {
        if let Some(last) = kept_indices.last_mut() {
            *last = index;
        }
    }
    let kept = kept_indices.iter().copied().collect::<HashSet<_>>();
    let selected = kept_indices
        .iter()
        .filter_map(|index| tools.get(*index).cloned())
        .collect();
    let mut omitted_tool_names = Vec::new();
    let mut additional_omitted_tools = 0;
    for (index, tool) in tools.iter().enumerate() {
        if kept.contains(&index) {
            continue;
        }
        if omitted_tool_names.len() < super::provider_tool_limits::MAX_CAPACITY_DIAGNOSTIC_ITEMS {
            omitted_tool_names.push(definition_name(tool).unwrap_or("unknown_tool").to_string());
        } else {
            additional_omitted_tools += 1;
        }
    }
    ActiveDefinitions {
        tools: selected,
        omitted_tool_names,
        additional_omitted_tools,
    }
}

pub fn append_capacity_notice(
    tools: &mut [Value],
    omitted_plugins: &[String],
    omitted_tools: &[String],
    additional_omitted_tools: usize,
) {
    if omitted_plugins.is_empty() && omitted_tools.is_empty() && additional_omitted_tools == 0 {
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
    let shown_plugins = omitted_plugins
        .iter()
        .take(super::provider_tool_limits::MAX_CAPACITY_DIAGNOSTIC_ITEMS)
        .cloned()
        .collect::<Vec<_>>()
        .join(", ");
    let remaining_plugins = omitted_plugins
        .len()
        .saturating_sub(super::provider_tool_limits::MAX_CAPACITY_DIAGNOSTIC_ITEMS);
    let plugin_notice = if shown_plugins.is_empty() {
        String::new()
    } else {
        let suffix = if remaining_plugins > 0 {
            format!(" (+{remaining_plugins} additional plugins)")
        } else {
            String::new()
        };
        format!("\nUnavailable plugin units: {shown_plugins}{suffix}.")
    };
    let tool_notice = (!omitted_tools.is_empty() || additional_omitted_tools > 0).then(|| {
        let names = omitted_tools.join(", ");
        let suffix = if additional_omitted_tools > 0 {
            format!(" (+{additional_omitted_tools} additional tools)")
        } else {
            String::new()
        };
        format!("\nUnavailable Beaver tools: {names}{suffix}.")
    });
    let notice = format!(
        "\n\nProvider tool limit reached.{plugin_notice}{}",
        tool_notice.unwrap_or_default()
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
