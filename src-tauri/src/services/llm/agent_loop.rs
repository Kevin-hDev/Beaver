use super::agent_loop_compression::{LastCounts, LoopCompression};
use super::agent_loop_request::ApiRequestParams;
use super::agent_loop_tools;
use crate::services::agent_local::agent_loop_finish;
use crate::services::agent_local::agent_loop_limits::MAX_TURNS;
use crate::services::agent_local::agent_loop_plan;
use crate::services::agent_local::circuit_breaker;
use crate::services::agent_local::stream_events::AgentEventEmitter;
use crate::services::agent_local::subagent_orchestration;
use crate::services::agent_local::tool_executor;
use crate::services::agent_local::types_ollama::ChatMessage;
use crate::services::agent_local::write_guard_registry;
use crate::services::token_counting;
use std::path::PathBuf;
use tokio_util::sync::CancellationToken;

pub async fn run_agent_loop(
    on_event: &AgentEventEmitter,
    provider_id: &str,
    model: &str,
    messages: &mut Vec<ChatMessage>,
    tools: &[serde_json::Value],
    think: bool,
    reasoning_mode: Option<&str>,
    working_dir: PathBuf,
    session_id: String,
    request_id: String,
    parent_message_inbox: Option<
        std::sync::Arc<crate::services::agent_local::parent_message_inbox::ParentMessageInbox>,
    >,
    cancel: CancellationToken,
    native_context: u64,
    configured_context: u64,
    permission_mode: &str,
    plan_mode_active: bool,
) -> Result<u32, String> {
    let (mut total_eval, mut total_prompt) = (Some(0), Some(0));
    let (mut last_prompt, mut last_eval) = (None, None);
    let start = std::time::Instant::now();
    let mut breaker = circuit_breaker::CircuitBreaker::new();
    let write_guard_arc = write_guard_registry::lock(&session_id).await;
    let mut write_guard = write_guard_arc.lock().await;
    let mut plan_repairs = 0;
    let mut subagents = subagent_orchestration::ParentSubagentOrchestrator::with_parent_inbox(
        &session_id,
        parent_message_inbox,
    )
    .await;
    let compression = LoopCompression {
        on_event,
        provider_id,
        model,
        session_id: &session_id,
        request_id: &request_id,
        native_context,
        configured_context,
        working_dir: &working_dir,
    };
    for turn in 0..MAX_TURNS {
        if cancel.is_cancelled() {
            return Err("Annulé".to_string());
        }
        let request_output = super::agent_loop_request::run(ApiRequestParams {
            on_event,
            messages,
            provider_id,
            model,
            tools,
            think,
            reasoning_mode,
            session_id: &session_id,
            request_id: &request_id,
            cancel: cancel.clone(),
            configured_context,
            plan_mode_active,
            turn,
            subagents: &mut subagents,
        })
        .await?;
        let interrupted = request_output.interrupted;
        let plan_active = request_output.plan_active;
        let result = request_output.result;
        if interrupted {
            crate::services::agent_local::stream_buffer::finalize_interrupted_content(
                on_event,
                &result,
                plan_active,
            );
            compression
                .handle_interrupted(
                    messages,
                    &result,
                    LastCounts::new(&mut last_prompt, &mut last_eval),
                    cancel.clone(),
                )
                .await?;
            continue;
        }
        token_counting::add_real_count(&mut total_eval, result.eval_count);
        token_counting::add_real_count(&mut total_prompt, result.prompt_tokens);
        last_prompt = result.prompt_tokens;
        last_eval = result.eval_count;
        match agent_loop_plan::check_result(
            on_event,
            messages,
            &session_id,
            &request_id,
            &result,
            plan_active,
            plan_repairs,
        )
        .await
        {
            agent_loop_plan::PlanLoopAction::Accept => plan_repairs = 0,
            agent_loop_plan::PlanLoopAction::Retry => {
                plan_repairs += 1;
                continue;
            }
            agent_loop_plan::PlanLoopAction::Stop(message) => return Err(message.to_string()),
        }
        subagents
            .finalize_content_phase(on_event, &result, plan_active)
            .await;
        messages.push(super::agent_loop_message::build_for_plan(
            &result,
            plan_active,
        ));
        compression
            .try_run_and_reset(messages, &mut last_prompt, &mut last_eval, cancel.clone())
            .await;
        if result.tool_calls.is_empty() {
            if subagents
                .continue_after_no_tool_turn(
                    on_event,
                    messages,
                    cancel.clone(),
                    turn + 1 < MAX_TURNS,
                )
                .await?
            {
                continue;
            }
            break;
        }
        let control_only = agent_loop_tools::prepare_tool_batch(
            &session_id,
            &request_id,
            &result.tool_calls,
            &working_dir,
            turn,
            &mut breaker,
        )
        .await?;
        let tool_compression = (!control_only).then(|| {
            compression.tool_compression(
                token_counting::sum_real_counts(last_prompt, last_eval),
                cancel.clone(),
            )
        });
        let compressed_during_tools = tool_executor::run_tools(
            on_event,
            messages,
            &result.tool_calls,
            &working_dir,
            permission_mode,
            &session_id,
            &request_id,
            cancel.clone(),
            &mut write_guard,
            plan_active,
            &result.tool_call_ids,
            tool_compression.as_ref(),
        )
        .await;
        subagents
            .wait_after_tool_batch(control_only, messages, cancel.clone())
            .await?;
        let compressed_after_tools = compression
            .after_tools(
                messages,
                compressed_during_tools,
                &mut last_prompt,
                &mut last_eval,
                cancel.clone(),
            )
            .await;
        agent_loop_finish::emit_turn_end(on_event, compressed_after_tools);
    }
    Ok(agent_loop_finish::finish(
        on_event,
        (total_eval, total_prompt, last_prompt, last_eval),
        start,
        (&session_id, &request_id),
        None,
    )
    .await)
}
