use super::types_tools::ToolResult;
use super::tool_dispatcher_route::{dynamic_route, is_chat_tool};
use super::tool_dispatch_trace::DispatchTrace;
use super::tool_result_contract::ToolErrorCategory;
use serde_json::Value;
use std::path::Path;
use tokio_util::sync::CancellationToken;

#[cfg(test)]
pub async fn dispatch(
    tool_name: &str,
    args: &Value,
    working_dir: &Path,
    session_id: &str,
    cancel: CancellationToken,
) -> ToolResult {
    dispatch_with_progress(
        tool_name,
        args,
        working_dir,
        DispatchTrace {
            session_id,
            request_id: None,
        },
        cancel,
        false,
        None,
    )
    .await
}

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
    let registered_dynamic =
        !chat_mode && crate::services::extensions::is_dynamic_tool(tool_name);
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
    if !super::tool_availability::available(
        enabled_by_settings,
        dynamic_tool,
        replacement,
    ) {
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
    let args = match validate_arguments(dynamic_tool, tool_name, args) {
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
    let before = super::tool_file_changes::direct_snapshot(tool_name, &args, working_dir);
    let mut result = if dynamic_tool {
        if crate::services::extensions::record_tool_invocation(tool_name).is_err() {
            ::log::warn!("[extensions] usage counter unavailable");
        }
        crate::services::extensions::dispatch_tool(tool_name, &args, working_dir)
            .await
            .unwrap_or_else(crate::services::extensions::unavailable_tool_result)
    } else {
        match super::memory_tool::dispatch_if_memory(tool_name, &args, working_dir, session_id).await
        {
            Some(result) => result,
            None => {
                super::tool_dispatcher::dispatch_inner(
                    tool_name,
                    &args,
                    working_dir,
                    trace,
                    cancel,
                    profile,
                    progress,
                )
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
    super::tool_dispatcher_finalize::finalize(result, tool_name, session_id, working_dir).await
}

pub(super) async fn finalize_result(
    result: ToolResult,
    tool_name: &str,
    session_id: &str,
    working_dir: &Path,
) -> ToolResult {
    super::tool_dispatcher_finalize::finalize(result, tool_name, session_id, working_dir).await
}

fn validate_arguments(dynamic_tool: bool, tool_name: &str, args: &Value) -> Result<Value, String> {
    if dynamic_tool {
        crate::services::extensions::validate_arguments(tool_name, args)
    } else {
        super::tool_validate::validate(tool_name, args)
    }
}

#[cfg(test)]
#[path = "tool_dispatcher_entry_tests.rs"]
mod tests;
