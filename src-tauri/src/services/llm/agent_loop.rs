#![expect(
    clippy::too_many_arguments,
    reason = "orchestration boundary keeps related runtime context explicit"
)]
use super::agent_loop_compression::{LastCounts, LoopCompression};
use super::{agent_loop_request::ApiRequestParams, agent_loop_tools};
use crate::services::agent_local::{
    agent_loop_finish, agent_loop_limits::MAX_TURNS, agent_loop_plan, circuit_breaker,
    context_usage_buckets::ContextUsageSeed, context_usage_runtime,
    generation_metrics::GenerationAggregate, stream_events::AgentEventEmitter,
    subagent_orchestration, types_ollama::ChatMessage, write_guard_registry,
};
use crate::services::token_counting;
use std::path::PathBuf;
use tokio_util::sync::CancellationToken;

pub async fn run_agent_loop(
    on_event: &AgentEventEmitter,
    provider_id: &str,
    fast_mode: super::fast_mode::FastModeRequest,
    model: &str,
    messages: &mut Vec<ChatMessage>,
    mut tools: crate::services::agent_local::extension_tool_set::ExtensionToolSet,
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
    context_usage_seed: ContextUsageSeed,
    continuation_target: Option<
        crate::services::reasoning_continuity::contract::ContinuationTarget,
    >,
    #[cfg(debug_assertions)] mut fixture_run: Option<
        &mut crate::services::reasoning_fixture_run::FixtureRunContext,
    >,
    mut journal: Option<
        &mut crate::services::agent_local::conversation_journal::ConversationJournal,
    >,
) -> Result<crate::services::agent_local::agent_loop_finish::CompletedStreamTurn, String> {
    let (mut total_eval, mut total_prompt) = (Some(0), Some(0));
    let (mut last_prompt, mut last_eval) = (None, None);
    let mut generation = GenerationAggregate::default();
    let mut breaker = circuit_breaker::CircuitBreaker::new();
    let write_guard_arc = write_guard_registry::lock(&session_id).await;
    let mut write_guard = write_guard_arc.lock().await;
    let mut plan_repairs = 0;
    let mut tool_result_previews =
        crate::services::agent_local::tool_artifact_preview::ToolResultPreviewBatch::default();
    #[cfg(debug_assertions)]
    let fixture_mode = fixture_run.is_some();
    // Packaged builds have no fixture runner, but share the tool-turn context.
    #[cfg(not(debug_assertions))]
    let fixture_mode = false;
    let mut subagents = subagent_orchestration::ParentSubagentOrchestrator::with_parent_inbox(
        &session_id,
        parent_message_inbox,
    )
    .await;
    let compression = LoopCompression {
        on_event,
        provider_id,
        fast_mode,
        model,
        session_id: &session_id,
        request_id: &request_id,
        native_context,
        configured_context,
        provider_tools: tools.active().to_vec(),
        chatbot: permission_mode == "chat",
        plan_mode_active,
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
            fast_mode,
            model,
            tools: tools.active(),
            think,
            reasoning_mode,
            session_id: &session_id,
            request_id: &request_id,
            cancel: cancel.clone(),
            configured_context,
            plan_mode_active,
            turn,
            subagents: &mut subagents,
            context_usage_seed,
            tool_result_previews: &tool_result_previews,
            continuation_target: continuation_target.clone(),
        })
        .await?;
        // A projection belongs to exactly one provider continuation; retries
        // already happened inside `run`, so compaction must not retain bytes.
        tool_result_previews.clear_after_projection();
        generation.merge(request_output.generation);
        let interrupted = request_output.interrupted;
        let plan_active = request_output.plan_active;
        let input_tokens = request_output.input_tokens;
        let result = request_output.result;
        if interrupted {
            if let Some(journal) = journal.as_deref_mut() {
                journal
                    .persist_partial(super::agent_loop_message::build_assistant_message(&result))
                    .await?;
            }
            crate::services::agent_local::stream_buffer::finalize_interrupted_content(
                on_event,
                &result,
                plan_active,
            );
            context_usage_runtime::emit_result(on_event, input_tokens, &result, configured_context);
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
        context_usage_runtime::emit_result(on_event, input_tokens, &result, configured_context);
        let assistant = super::agent_loop_message::build_for_plan(&result, plan_active);
        if let Some(journal) = journal.as_deref_mut() {
            journal.persist_assistant_step(&assistant).await?;
        }
        messages.push(assistant);
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
        let tool_turn = agent_loop_tools::run_tool_turn(agent_loop_tools::ToolTurnContext {
            on_event,
            messages,
            result: &result,
            working_dir: &working_dir,
            permission_mode,
            session_id: &session_id,
            request_id: &request_id,
            cancel: cancel.clone(),
            write_guard: &mut write_guard,
            plan_active,
            turn,
            breaker: &mut breaker,
            journal: journal.as_deref_mut(),
            tools: &mut tools,
            subagents: &mut subagents,
            compression: &compression,
            last_prompt: &mut last_prompt,
            last_eval: &mut last_eval,
            fixture_mode,
            #[cfg(debug_assertions)]
            fixture_run: fixture_run.as_deref_mut(),
        })
        .await?;
        tool_result_previews = tool_turn.previews;
        if tool_turn.stop {
            break;
        }
    }
    Ok(agent_loop_finish::finish(
        (total_eval, total_prompt, last_prompt, last_eval),
        generation,
        (&session_id, &request_id),
        None,
    )
    .await)
}
