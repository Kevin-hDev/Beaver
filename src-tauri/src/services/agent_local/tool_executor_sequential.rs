#![expect(clippy::too_many_arguments, reason = "orchestration boundary keeps related runtime context explicit")]
use crate::services::agent_local::stream_events::AgentEventEmitter;
use crate::services::agent_local::tool_hooks::{run_post_hooks, run_pre_hooks, PreHookDecision};
use crate::services::agent_local::types_ollama::ChatMessage;
use crate::services::agent_local::write_guard::WriteGuard;
use tokio_util::sync::CancellationToken;

use super::tool_executor_helpers::{
    check_write_guard, dispatch_or_interactive, post_record_read, post_record_write,
};
use super::tool_executor_compression::ToolCompression;
use super::tool_execution_outcome::ToolExecutionOutcome;
use super::tool_executor_sequential_support::{
    check_allowed, initial_validation, push_and_compress,
};

pub async fn run_sequential(
    on_event: &AgentEventEmitter,
    messages: &mut Vec<ChatMessage>,
    tool_calls: &[(String, serde_json::Value)],
    working_dir: &std::path::Path,
    session_id: &str,
    request_id: &str,
    cancel: CancellationToken,
    write_guard: &mut WriteGuard,
    plan_mode_active: bool,
    tool_call_ids: &[String],
    compression: Option<&ToolCompression<'_>>,
) -> ToolExecutionOutcome {
    let mut outcome = ToolExecutionOutcome::default();
    for (idx, (name, args)) in tool_calls.iter().enumerate() {
        let arg_summary = super::tool_executor_diagnostics::started(
            session_id,
            name,
            args,
            working_dir,
        )
        .await;
        let arg_summary = match initial_validation(
            session_id,
            request_id,
            name,
            args,
            working_dir,
            plan_mode_active,
            arg_summary,
        )
        .await
        {
            Ok(summary) => summary,
            Err(tr) => {
                outcome.merge(push_and_compress(
                    on_event,
                    messages,
                    name,
                    args,
                    working_dir,
                    tr,
                    idx,
                    tool_call_ids,
                    compression,
                )
                .await);
                continue;
            }
        };
        match run_pre_hooks(name, args) {
            PreHookDecision::Deny(msg) => {
                let tr = super::tool_executor_errors::permission(msg, "tool_hook_denied");
                super::tool_executor_diagnostics::completed(
                    session_id,
                    request_id,
                    name,
                    arg_summary,
                    &tr,
                )
                .await;
                outcome.merge(push_and_compress(
                    on_event,
                    messages,
                    name,
                    args,
                    working_dir,
                    tr,
                    idx,
                    tool_call_ids,
                    compression,
                )
                .await);
                continue;
            }
            PreHookDecision::Allow => {}
        }

        if let Err(msg) = check_write_guard(name, args, working_dir, write_guard) {
            let tr = super::tool_executor_errors::permission(msg, "write_guard_rejected");
            super::tool_executor_diagnostics::completed(
                session_id,
                request_id,
                name,
                arg_summary,
                &tr,
            )
            .await;
            outcome.merge(push_and_compress(
                on_event,
                messages,
                name,
                args,
                working_dir,
                tr,
                idx,
                tool_call_ids,
                compression,
            )
            .await);
            continue;
        }

        let allowed = check_allowed(on_event, name, args, session_id, cancel.clone()).await;
        let tr = if allowed {
            if let Err(msg) = super::tool_plan_guard::ensure_allowed_for_session(
                name,
                args,
                session_id,
                plan_mode_active,
            )
            .await
            {
                super::tool_executor_errors::permission(msg, "tool_not_allowed_in_plan")
            } else {
                dispatch_or_interactive(
                    on_event,
                    name,
                    args,
                    working_dir,
                    super::tool_dispatch_trace::DispatchTrace {
                        session_id,
                        request_id: Some(request_id),
                    },
                    cancel.clone(),
                    Some(idx),
                )
                .await
            }
        } else {
            super::tool_executor_errors::denied_or_cancelled(&cancel)
        };

        let tr = run_post_hooks(name, args, tr);
        post_record_read(name, args, working_dir, &tr, write_guard);
        post_record_write(name, args, working_dir, &tr, write_guard);
        super::tool_executor_diagnostics::completed(
            session_id,
            request_id,
            name,
            arg_summary,
            &tr,
        )
        .await;
        outcome.merge(push_and_compress(
            on_event,
            messages,
            name,
            args,
            working_dir,
            tr,
            idx,
            tool_call_ids,
            compression,
        )
        .await);
    }
    outcome
}
