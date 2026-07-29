use super::common::StreamMode;
use super::params::StreamTaskParams;
use crate::services::agent_local::agent_settings::AgentSettings;
use crate::services::agent_local::types_ollama::StreamEvent;
use crate::services::agent_local::{tool_catalog, tool_dispatcher};
use serde_json::Value;

pub(super) fn resolve(
    params: &StreamTaskParams,
    mode: &StreamMode,
    model_supports_tools: bool,
    settings: &AgentSettings,
    canonical_provider: &str,
) -> Vec<Value> {
    let definitions = if mode.is_chat {
        tool_dispatcher::get_chat_tool_definitions()
    } else if !model_supports_tools {
        vec![]
    } else if params.tools.is_empty() {
        tool_dispatcher::get_tool_definitions()
    } else {
        params.tools.clone()
    };
    let filtered =
        tool_catalog::filter_tool_definitions(definitions, &settings.enabled_optional_tools);
    let policy = super::tool_policy::apply(canonical_provider, &params.model, filtered);
    if policy.extensions_blocked {
        let _ = params.on_event.send(StreamEvent::Notice {
            message_key: super::tool_policy::GROQ_EXTENSIONS_NOTICE.to_string(),
        });
    }
    policy.tools
}

pub(super) fn todo_tools_enabled(enabled_tool_names: &[String]) -> bool {
    tool_catalog::has_any_tool(
        enabled_tool_names,
        &[
            "todo_write",
            "todo_history",
            "todo_pause",
            "todo_resume",
            "todo_delete",
            "agent_diagnostics",
        ],
    )
}
