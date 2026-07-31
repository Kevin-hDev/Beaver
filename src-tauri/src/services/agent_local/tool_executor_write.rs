use crate::services::agent_local::stream_events::AgentEventEmitter;
use crate::services::agent_local::tool_dispatcher;
use crate::services::agent_local::tool_hooks::{run_post_hooks, run_pre_hooks, PreHookDecision};
use crate::services::agent_local::types_tools::ToolResult;
use crate::services::agent_local::write_guard::WriteGuard;
use crate::services::agent_local::{permission_gate, permission_policy, sensitive_data};
use serde_json::Value;
use tokio_util::sync::CancellationToken;

use super::tool_executor_helpers::{check_write_guard, dispatch_or_interactive, post_record_write};

pub(super) async fn execute_write(
    on_event: &AgentEventEmitter,
    name: &str,
    args: &Value,
    working_dir: &std::path::Path,
    mode: &str,
    write_guard: &mut WriteGuard,
    session_id: &str,
    cancel: CancellationToken,
    plan_mode_active: bool,
    tool_call_index: Option<usize>,
) -> ToolResult {
    if mode == "chat" {
        return tool_dispatcher::dispatch_for_mode(
            name,
            args,
            working_dir,
            session_id,
            cancel,
            true,
        )
        .await;
    }
    if let Err(msg) =
        super::tool_plan_guard::ensure_allowed_for_session(name, args, session_id, plan_mode_active)
            .await
    {
        return tool_dispatcher::enrich_error(ToolResult::err(msg), name);
    }
    match run_pre_hooks(name, args) {
        PreHookDecision::Deny(msg) => {
            return tool_dispatcher::enrich_error(ToolResult::err(msg), name);
        }
        PreHookDecision::Allow => {}
    }

    let memory_write =
        match super::memory_tool::write_authorization(name, args, working_dir, session_id) {
            Ok(authorization) => authorization,
            Err(message) => return ToolResult::err(message),
        };
    if memory_write == Some(false) {
        return ToolResult::err("Cette écriture mémoire n'est pas autorisée pour ce tour.");
    }

    if memory_write == Some(true) {
        permission_gate::log_diagnostic(
            "memory_write_authorized",
            Some(name),
            Some("memory_policy"),
        );
    } else if permission_policy::uses_auto_bypass(mode) {
        permission_gate::log_diagnostic("auto_bypass", Some(name), Some(mode));
    } else if permission_policy::requires_sensitive_bash_prompt(mode, name, args) {
        let safe_args = sensitive_data::redact_json(args);
        if !request_once(on_event, name, &safe_args, cancel.clone()).await {
            return ToolResult::err("L'utilisateur a refusé cette action.");
        }
    } else if permission_gate::requires_permission(name, args)
        && !permission_gate::is_allowed(session_id, name).await
    {
        match permission_gate::request(on_event, name, args, cancel.clone()).await {
            permission_gate::PermissionDecision::Allow => {}
            permission_gate::PermissionDecision::AllowSession => {
                permission_gate::mark_allowed(session_id, name).await;
            }
            permission_gate::PermissionDecision::Deny => {
                return ToolResult::err("L'utilisateur a refusé cette action.");
            }
        }
    }

    let tr = match check_write_guard(name, args, working_dir, write_guard) {
        Err(msg) => tool_dispatcher::enrich_error(ToolResult::err(msg), name),
        Ok(()) => {
            if cancel.is_cancelled() {
                ToolResult::err("Annulé.")
            } else if let Err(msg) = super::tool_plan_guard::ensure_allowed_for_session(
                name,
                args,
                session_id,
                plan_mode_active,
            )
            .await
            {
                tool_dispatcher::enrich_error(ToolResult::err(msg), name)
            } else {
                dispatch_or_interactive(
                    on_event,
                    name,
                    args,
                    working_dir,
                    session_id,
                    cancel.clone(),
                    tool_call_index,
                )
                .await
            }
        }
    };
    let tr = run_post_hooks(name, args, tr);
    // Enregistre le fichier écrit comme "déjà vu" pour ne pas bloquer le tour suivant.
    post_record_write(name, args, working_dir, &tr, write_guard);
    tr
}

async fn request_once(
    on_event: &AgentEventEmitter,
    name: &str,
    args: &Value,
    cancel: CancellationToken,
) -> bool {
    match permission_gate::request(on_event, name, args, cancel).await {
        permission_gate::PermissionDecision::Allow
        | permission_gate::PermissionDecision::AllowSession => true,
        permission_gate::PermissionDecision::Deny => false,
    }
}
