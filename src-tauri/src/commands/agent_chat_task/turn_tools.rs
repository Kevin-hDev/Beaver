use super::common::StreamMode;
use super::params::StreamTaskParams;
use crate::services::agent_local::extension_tool_set::{ExtensionToolSet, PrepareContext};
use crate::services::agent_local::tool_catalog;
use crate::services::agent_local::types_ollama::ChatMessage;

pub(super) async fn prepare_extensions(
    params: &StreamTaskParams,
    mode: &StreamMode,
    provider: &str,
    context_window: u64,
    definitions: Vec<serde_json::Value>,
    fixture_mode: bool,
) -> Result<(ExtensionToolSet, Vec<String>), String> {
    let tools = if fixture_mode || mode.is_chat {
        ExtensionToolSet::passthrough(definitions)
    } else {
        ExtensionToolSet::prepare(
            definitions,
            PrepareContext {
                session_id: &params.session_id,
                provider,
                model: &params.model,
                context_window,
                preserve_dynamic_tools: preserve_explicit_dynamic_tools(
                    !params.tools.is_empty(),
                    params.subagent_profile.is_some(),
                ),
            },
        )
        .await?
    };
    let names = tool_catalog::tool_names(tools.active());
    tools
        .report_prepared(&params.on_event, &params.session_id, &params.request_id)
        .await?;
    Ok((tools, names))
}

pub(super) async fn resolve_plan_mode(
    params: &StreamTaskParams,
    enabled_tool_names: &[String],
    fixture_mode: bool,
) -> bool {
    if fixture_mode {
        return false;
    }
    let requested = match params.plan_mode {
        Some(value) => value,
        None => crate::services::agent_local::tool_plan::is_enabled(&params.session_id).await,
    };
    plan_mode_active(requested, enabled_tool_names)
}

pub(super) async fn append_todo_reminder(
    messages: &mut [ChatMessage],
    session_id: &str,
    enabled_tool_names: &[String],
    fixture_mode: bool,
) {
    if !fixture_mode && todo_tools_enabled(enabled_tool_names) {
        crate::services::agent_local::tool_todo::append_session_reminder(messages, session_id)
            .await;
    }
}

fn plan_mode_active(requested: bool, enabled_tool_names: &[String]) -> bool {
    requested && tool_catalog::has_plan_tools(enabled_tool_names)
}

fn todo_tools_enabled(enabled_tool_names: &[String]) -> bool {
    tool_catalog::has_any_tool(
        enabled_tool_names,
        &[
            "todo_write",
            "todo_history",
            "todo_pause",
            "todo_resume",
            "todo_delete",
        ],
    )
}

fn preserve_explicit_dynamic_tools(has_explicit_tools: bool, is_subagent: bool) -> bool {
    has_explicit_tools && !is_subagent
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plan_mode_requires_both_request_and_published_tool() {
        let plan_tools = vec!["plan_mode".to_string()];
        assert!(plan_mode_active(true, &plan_tools));
        assert!(!plan_mode_active(false, &plan_tools));
        assert!(!plan_mode_active(true, &["read_file".to_string()]));
    }

    #[test]
    fn explicit_subagent_tools_do_not_bypass_extension_discovery() {
        assert!(preserve_explicit_dynamic_tools(true, false));
        assert!(!preserve_explicit_dynamic_tools(true, true));
        assert!(!preserve_explicit_dynamic_tools(false, true));
    }
}
