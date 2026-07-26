use super::tool_hooks::{run_post_hooks, run_pre_hooks, PreHookDecision};
use super::types_tools::ToolResult;
use serde_json::Value;
use tokio_util::sync::CancellationToken;

pub(super) async fn launch(
    args: &Value,
    session_id: &str,
    plan_mode_active: bool,
    cancel: CancellationToken,
) -> Result<super::tool_dispatcher_delegate::PendingDelegate, ToolResult> {
    let tool = super::tool_executor_delegate_batch::DELEGATE_TOOL;
    if let Err(msg) = super::tool_plan_guard::ensure_allowed_for_session(
        tool,
        args,
        session_id,
        plan_mode_active,
    )
    .await
    {
        return Err(super::tool_dispatcher::enrich_error(
            ToolResult::err(msg),
            tool,
        ));
    }
    if super::tool_catalog::is_optional_tool(tool)
        && !super::agent_settings::is_tool_enabled(tool).await
    {
        return Err(ToolResult::err("Outil désactivé dans les paramètres."));
    }
    match run_pre_hooks(tool, args) {
        PreHookDecision::Deny(msg) => Err(super::tool_dispatcher::enrich_error(
            ToolResult::err(msg),
            tool,
        )),
        PreHookDecision::Allow => {
            super::tool_dispatcher_delegate::spawn_delegate(args, session_id, cancel)
                .await
                .map_err(|result| run_post_hooks(tool, args, result))
        }
    }
}
