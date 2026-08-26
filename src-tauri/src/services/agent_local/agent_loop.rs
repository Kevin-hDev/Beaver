#![expect(clippy::too_many_arguments, reason = "orchestration boundary keeps related runtime context explicit")]
use super::{
    agent_loop_compression::{LastCounts, LoopCompression},
    agent_loop_ollama_request::OllamaRequestParams,
    agent_loop_limits::MAX_TURNS, agent_loop_plan, agent_loop_support, circuit_breaker,
    context_usage_buckets::ContextUsageSeed, context_usage_runtime,
    stream_events::AgentEventEmitter, tool_executor,
    types_ollama::{ChatMessage, OllamaThink}, write_guard_registry,
};
use crate::services::token_counting;
pub async fn run_agent_loop(
    on_event: &AgentEventEmitter,
    messages: &mut Vec<ChatMessage>,
    model: &str,
    mut tools: super::extension_tool_set::ExtensionToolSet,
    think: OllamaThink,
    working_dir: std::path::PathBuf,
    session_id: String,
    request_id: String,
    parent_message_inbox: Option<std::sync::Arc<super::parent_message_inbox::ParentMessageInbox>>,
    cancel: tokio_util::sync::CancellationToken,
    native_context: u64,
    configured_context: u64,
    permission_mode: &str,
    plan_mode_active: bool,
    context_usage_seed: ContextUsageSeed,
    capture_reasoning: bool,
    mut journal: Option<&mut super::conversation_journal::ConversationJournal>,
) -> Result<super::agent_loop_finish::CompletedStreamTurn, String> {
    let (mut total_eval, mut total_prompt) = (Some(0), Some(0));
    let (mut last_prompt, mut last_eval) = (None, None);
    let mut generation = super::generation_metrics::GenerationAggregate::default();
    let mut breaker = circuit_breaker::CircuitBreaker::new();
    let write_guard_arc = write_guard_registry::lock(&session_id).await;
    let mut write_guard = write_guard_arc.lock().await;
    let mut plan_repairs = 0;
    let mut subagents =
        agent_loop_support::prepare_subagents(&session_id, parent_message_inbox).await;
    let compression = LoopCompression {
        on_event,
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
        let request_output = super::agent_loop_ollama_request::run(OllamaRequestParams {
            on_event,
            messages,
            model,
            tools: tools.active(),
            think: &think,
            working_dir: &working_dir,
            session_id: &session_id,
            request_id: &request_id,
            cancel: cancel.clone(),
            configured_context,
            plan_mode_active,
            chat_mode: permission_mode == "chat",
            turn,
            subagents: &mut subagents,
            context_usage_seed,
            capture_reasoning,
        })
        .await?;
        generation.merge(request_output.generation);
        let eager_handle = request_output.eager_handle;
        let interrupted = request_output.interrupted;
        let plan_active = request_output.plan_active;
        let input_tokens = request_output.input_tokens;
        let result = request_output.result;
        if interrupted {
            if let Some(journal) = journal.as_deref_mut() {
                journal
                    .persist_partial(agent_loop_support::build_assistant_message(&result))
                    .await?;
            }
            super::stream_buffer::finalize_interrupted_content(on_event, &result, plan_active);
            context_usage_runtime::emit_result(on_event, input_tokens, &result, configured_context);
            eager_handle.abort();
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
                eager_handle.abort();
                continue;
            }
            agent_loop_plan::PlanLoopAction::Stop(message) => {
                eager_handle.abort();
                agent_loop_support::decharge_gpu(model).await;
                return Err(message.to_string());
            }
        }
        subagents
            .finalize_content_phase(on_event, &result, plan_active)
            .await;
        context_usage_runtime::emit_result(on_event, input_tokens, &result, configured_context);
        let assistant = agent_loop_support::build_for_plan(&result, plan_active);
        if let Some(journal) = journal.as_deref_mut() {
            journal.persist_assistant_step(&assistant).await?;
        }
        messages.push(assistant);
        compression
            .try_run_and_reset(messages, &mut last_prompt, &mut last_eval, cancel.clone())
            .await;
        if result.tool_calls.is_empty() {
            eager_handle.abort();
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
        if turn == MAX_TURNS - 1 {
            eager_handle.abort();
            agent_loop_support::ensure_more_turns(turn, model).await?;
        }
        if let Err(msg) = breaker.check(&result.tool_calls) {
            eager_handle.abort();
            agent_loop_support::decharge_gpu(model).await;
            return Err(msg);
        }
        let control_only = super::subagent_tool_control::is_control_only(&result.tool_calls);
        let eager_results = eager_handle.await.unwrap_or_default();
        let tool_start = messages.len();
        let tool_outcome = tool_executor::run_tools_with_eager(
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
            Some(eager_results),
            &[],
            None,
        )
        .await;
        let compressed_during_tools = tool_outcome.compressed;
        let tool_end = messages.len();
        if let Some(journal) = journal.as_deref_mut() {
            journal
                .persist_tool_results(&messages[tool_start..tool_end])
                .await?;
        }
        let stop_after_tools = tool_outcome.apply_follow_ups(messages);
        super::extension_tool_set::refresh_and_record(&mut tools, &session_id, &request_id).await?;
        subagents
            .wait_after_tool_batch(control_only, messages, cancel.clone())
            .await?;
        let counts = LastCounts::new(&mut last_prompt, &mut last_eval);
        compression
            .finish_tools(messages, compressed_during_tools, counts, cancel.clone())
            .await;
        if stop_after_tools {
            break;
        }
    }
    Ok(super::agent_loop_finish::finish(
        (total_eval, total_prompt, last_prompt, last_eval),
        generation,
        (&session_id, &request_id),
        Some(model),
    )
    .await)
}
