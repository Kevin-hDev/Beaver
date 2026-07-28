use std::collections::HashSet;

use serde_json::Value;

use super::types_ollama::ChatMessage;

const MAX_CONTEXT_QUERY_CHARS: usize = 4096;
const MAX_CONTEXT_MESSAGES: usize = 4;

pub struct ExtensionToolSet {
    active: Vec<Value>,
    available: Vec<Value>,
}

impl ExtensionToolSet {
    pub fn prepare(
        tools: Vec<Value>,
        messages: &[ChatMessage],
        preserve_dynamic_tools: bool,
    ) -> Self {
        let selected = if preserve_dynamic_tools {
            tools
                .iter()
                .filter_map(definition_name)
                .filter(|name| crate::services::extensions::is_dynamic_tool(name))
                .map(str::to_string)
                .collect()
        } else {
            crate::services::extensions::select_plugin_tools(
                &recent_user_context(messages),
                crate::services::extensions::MAX_SELECTED_TOOLS,
            )
            .into_iter()
            .collect::<HashSet<_>>()
        };
        let mut active = Vec::with_capacity(tools.len());
        let mut available = Vec::new();
        for tool in tools {
            let Some(name) = definition_name(&tool) else {
                continue;
            };
            if crate::services::extensions::is_dynamic_tool(name)
                && !crate::services::extensions::is_replacement(name)
                && !selected.contains(name)
            {
                available.push(tool);
            } else {
                active.push(tool);
            }
        }
        if available.is_empty() {
            active.retain(|tool| {
                definition_name(tool) != Some(crate::services::extensions::SEARCH_TOOL_NAME)
            });
        }
        Self { active, available }
    }

    pub fn active(&self) -> &[Value] {
        &self.active
    }

    pub fn selected_extension_names(&self) -> Vec<String> {
        self.active
            .iter()
            .filter_map(definition_name)
            .filter(|name| crate::services::extensions::is_dynamic_tool(name))
            .map(str::to_string)
            .collect()
    }

    pub fn expand_from_calls(&mut self, calls: &[(String, Value)]) -> Vec<String> {
        let mut requested = Vec::new();
        for (_, args) in calls
            .iter()
            .filter(|(name, _)| name == crate::services::extensions::SEARCH_TOOL_NAME)
        {
            let Some(query) = args.get("query").and_then(Value::as_str) else {
                continue;
            };
            if query.chars().count() > crate::services::extensions::MAX_SEARCH_QUERY_CHARS {
                continue;
            }
            let remaining = crate::services::extensions::MAX_SELECTED_TOOLS
                .saturating_sub(self.active_extension_count() + requested.len());
            let selected = crate::services::extensions::discover_plugin_tools(query, remaining);
            requested.extend(selected);
        }
        requested.sort();
        requested.dedup();
        let mut added = Vec::new();
        for name in requested {
            if self.active_extension_count()
                >= crate::services::extensions::MAX_SELECTED_TOOLS
            {
                break;
            }
            let Some(index) = self
                .available
                .iter()
                .position(|tool| definition_name(tool) == Some(name.as_str()))
            else {
                continue;
            };
            self.active.push(self.available.remove(index));
            added.push(name);
        }
        if self.available.is_empty() {
            self.active.retain(|tool| {
                definition_name(tool) != Some(crate::services::extensions::SEARCH_TOOL_NAME)
            });
        }
        added
    }

    fn active_extension_count(&self) -> usize {
        self.active
            .iter()
            .filter_map(definition_name)
            .filter(|name| crate::services::extensions::is_dynamic_tool(name))
            .count()
    }
}

pub async fn expand_and_record(
    tools: &mut ExtensionToolSet,
    calls: &[(String, Value)],
    session_id: &str,
    request_id: &str,
) {
    let added = tools.expand_from_calls(calls);
    if !added.is_empty() {
        super::stream_diagnostics::record_extension_tools(
            session_id,
            request_id,
            "extension_tools_expanded",
            &added,
        )
        .await;
    }
}

pub async fn record_initial(
    tools: &ExtensionToolSet,
    session_id: &str,
    request_id: &str,
) {
    let names = tools.selected_extension_names();
    if !names.is_empty() {
        super::stream_diagnostics::record_extension_tools(
            session_id,
            request_id,
            "extension_tools_selected",
            &names,
        )
        .await;
    }
}

fn recent_user_context(messages: &[ChatMessage]) -> String {
    let mut remaining = MAX_CONTEXT_QUERY_CHARS;
    let mut parts = Vec::new();
    for message in messages
        .iter()
        .rev()
        .filter(|message| message.role == "user")
        .take(MAX_CONTEXT_MESSAGES)
    {
        if remaining == 0 {
            break;
        }
        let part = message.content.chars().take(remaining).collect::<String>();
        remaining = remaining.saturating_sub(part.chars().count());
        parts.push(part);
    }
    parts
        .join("\n")
        .chars()
        .take(MAX_CONTEXT_QUERY_CHARS)
        .collect()
}

fn definition_name(definition: &Value) -> Option<&str> {
    definition.pointer("/function/name").and_then(Value::as_str)
}

#[cfg(test)]
#[path = "extension_tool_set_tests.rs"]
mod tests;
