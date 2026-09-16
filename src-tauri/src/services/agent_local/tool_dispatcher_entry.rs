use super::tool_dispatch_trace::DispatchTrace;
use super::tool_dispatcher_route::{dynamic_route, is_chat_tool};
use super::tool_dispatcher_finalize::finalize as finalize_result;
use super::tool_result_contract::ToolErrorCategory;
use super::types_tools::ToolResult;
use super::extension_tool_authority::ToolDispatchAuthority;
use serde_json::Value;
use std::path::Path;
use tokio_util::sync::CancellationToken;

pub async fn dispatch_for_mode(
    tool_name: &str,
    args: &Value,
    working_dir: &Path,
    session_id: &str,
    request_id: Option<&str>,
    cancel: CancellationToken,
    chat_mode: bool,
) -> ToolResult {
    dispatch_with_progress(
        tool_name,
        args,
        working_dir,
        DispatchTrace {
            session_id,
            request_id,
            tool_call_id: None,
        },
        cancel,
        chat_mode,
        None,
    )
    .await
}

pub async fn dispatch_with_progress(
    tool_name: &str,
    args: &Value,
    working_dir: &Path,
    trace: DispatchTrace<'_>,
    cancel: CancellationToken,
    chat_mode: bool,
    progress: Option<super::tool_bash_progress::ShellProgress>,
) -> ToolResult {
    dispatch_inner_entry(
        tool_name,
        args,
        working_dir,
        trace,
        cancel,
        chat_mode,
        progress,
        None,
    )
    .await
}

#[expect(
    clippy::too_many_arguments,
    reason = "dispatch boundary keeps the authorized turn context explicit"
)]
pub async fn dispatch_authorized_with_progress(
    tool_name: &str,
    args: &Value,
    working_dir: &Path,
    trace: DispatchTrace<'_>,
    cancel: CancellationToken,
    chat_mode: bool,
    progress: Option<super::tool_bash_progress::ShellProgress>,
    authority: ToolDispatchAuthority,
) -> ToolResult {
    dispatch_inner_entry(
        tool_name,
        args,
        working_dir,
        trace,
        cancel,
        chat_mode,
        progress,
        Some(authority),
    )
    .await
}

#[allow(clippy::too_many_arguments)]
async fn dispatch_inner_entry(
    tool_name: &str,
    args: &Value,
    working_dir: &Path,
    trace: DispatchTrace<'_>,
    cancel: CancellationToken,
    chat_mode: bool,
    progress: Option<super::tool_bash_progress::ShellProgress>,
    authority: Option<ToolDispatchAuthority>,
) -> ToolResult {
    let session_id = trace.session_id;
    if chat_mode && !is_chat_tool(tool_name) {
        return finalize_result(
            ToolResult::error(
                "Outil indisponible dans ce mode.",
                "tool_unavailable_in_mode",
                ToolErrorCategory::Unavailable,
                false,
            ),
            tool_name,
            session_id,
            working_dir,
        )
        .await;
    }
    let registered_dynamic = !chat_mode && crate::services::extensions::is_dynamic_tool(tool_name);
    let replacement = crate::services::extensions::is_replacement(tool_name);
    let active_dynamic = if registered_dynamic {
        match super::extension_session_plugins::is_tool_active(session_id, tool_name).await {
            Ok(active) => active,
            Err(_) => {
                return finalize_result(
                    crate::services::extensions::unavailable_tool_result(),
                    tool_name,
                    session_id,
                    working_dir,
                )
                .await
            }
        }
    } else {
        false
    };
    let dynamic_tool = match dynamic_route(registered_dynamic, active_dynamic, replacement) {
        Ok(dynamic) => dynamic,
        Err(_) => {
            return finalize_result(
                crate::services::extensions::unavailable_tool_result(),
                tool_name,
                session_id,
                working_dir,
            )
            .await
        }
    };
    let enabled_by_settings = !super::tool_catalog::is_optional_tool(tool_name)
        || super::agent_settings::is_tool_enabled(tool_name).await;
    if !super::tool_availability::available(enabled_by_settings, dynamic_tool, replacement) {
        return finalize_result(
            ToolResult::error(
                "Outil désactivé dans les paramètres.",
                "tool_disabled",
                ToolErrorCategory::Permission,
                false,
            ),
            tool_name,
            session_id,
            working_dir,
        )
        .await;
    }
    let profile = match super::subagent_tool_guard::validate_for_session(
        session_id,
        tool_name,
        args,
        working_dir,
    )
    .await
    {
        Ok(profile) => profile,
        Err(msg) => {
            return finalize_result(
                ToolResult::error(
                    msg,
                    "tool_not_allowed_for_session",
                    ToolErrorCategory::Permission,
                    false,
                ),
                tool_name,
                session_id,
                working_dir,
            )
            .await
        }
    };
    let args = match super::tool_dispatcher_validation::validate(dynamic_tool, tool_name, args) {
        Ok(cleaned) => cleaned,
        Err(msg) => {
            return finalize_result(
                ToolResult::error(
                    format!("[{tool_name}] {msg}"),
                    "invalid_tool_arguments",
                    ToolErrorCategory::Validation,
                    false,
                ),
                tool_name,
                session_id,
                working_dir,
            )
            .await
        }
    };
    let event = super::tool_dispatcher_events::start(
        authority.is_some(),
        trace,
        tool_name,
        &args,
        working_dir,
    );
    let result = super::tool_dispatcher_execute::execute(
        tool_name,
        args,
        working_dir,
        trace,
        cancel,
        profile,
        progress,
        dynamic_tool,
        authority,
    )
    .await;
    super::tool_dispatcher_events::finish(event, trace, tool_name, &result);
    result
}

#[cfg(test)]
#[path = "tool_dispatcher_entry_tests.rs"]
mod tests;
