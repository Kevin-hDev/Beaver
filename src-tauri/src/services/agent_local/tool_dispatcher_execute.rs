use super::extension_tool_authority::ToolDispatchAuthority;
use super::subagent_tool_profile::SubagentToolProfile;
use super::tool_dispatch_trace::DispatchTrace;
use super::types_tools::ToolResult;
use serde_json::Value;
use std::path::Path;
use tokio_util::sync::CancellationToken;

#[allow(clippy::too_many_arguments)]
pub(super) async fn execute(
    tool_name: &str,
    args: Value,
    working_dir: &Path,
    trace: DispatchTrace<'_>,
    cancel: CancellationToken,
    profile: Option<SubagentToolProfile>,
    progress: Option<super::tool_bash_progress::ShellProgress>,
    dynamic_tool: bool,
    authority: Option<ToolDispatchAuthority>,
) -> ToolResult {
    let before = super::tool_file_changes::direct_snapshot(tool_name, &args, working_dir);
    let mut result = if dynamic_tool {
        if crate::services::extensions::record_tool_invocation(tool_name).is_err() {
            log::warn!("[extensions] usage counter unavailable");
        }
        let Some(authority) = authority else {
            return crate::services::extensions::unavailable_tool_result();
        };
        super::extension_tool_authority::dispatch(
            tool_name,
            &args,
            working_dir,
            trace,
            cancel.clone(),
            profile,
            &authority,
        )
        .await
    } else {
        match super::memory_tool::dispatch_if_memory(
            tool_name,
            &args,
            working_dir,
            trace.session_id,
        )
        .await
        {
            Some(result) => result,
            None => {
                Box::pin(super::tool_dispatcher::dispatch_inner(
                    tool_name,
                    &args,
                    working_dir,
                    trace,
                    cancel,
                    profile,
                    progress,
                ))
                .await
            }
        }
    };
    if let Some(change) = before.and_then(super::tool_file_changes::direct_change) {
        if result.affected_paths().is_empty() {
            result.affected_paths_mut().push(change.path.clone());
        }
        result.file_changes_mut().push(change);
    }
    super::tool_dispatcher_finalize::finalize(result, tool_name, trace.session_id, working_dir).await
}
