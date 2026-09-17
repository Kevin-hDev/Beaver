use super::common::StreamMode;
use super::params::StreamTaskParams;
use crate::services::agent_local::agent_settings::AgentSettings;
use crate::services::agent_local::{tool_catalog, tool_dispatcher};

pub(super) fn resolve_tools(
    params: &StreamTaskParams,
    mode: &StreamMode,
    settings: &AgentSettings,
) -> Vec<serde_json::Value> {
    let definitions = definitions_for_mode(mode.is_chat, &params.tools);
    tool_catalog::filter_tool_definitions(definitions, &settings.enabled_optional_tools)
}

pub(super) fn definitions_for_mode(
    is_chat: bool,
    requested: &[serde_json::Value],
) -> Vec<serde_json::Value> {
    if is_chat {
        tool_dispatcher::get_chat_tool_definitions()
    } else if !requested.is_empty() {
        requested.to_vec()
    } else {
        tool_dispatcher::get_tool_definitions()
    }
}
