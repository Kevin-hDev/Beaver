#![expect(
    clippy::too_many_arguments,
    reason = "orchestration boundary keeps related runtime context explicit"
)]
use super::{
    agent_loop_compression::LoopCompression,
    agent_loop_limits::MAX_TURNS,
    agent_loop_ollama_request::OllamaRequestParams,
    agent_loop_plan, agent_loop_support, circuit_breaker,
    context_usage_buckets::ContextUsageSeed,
    context_usage_runtime,
    stream_events::AgentEventEmitter,
    types_ollama::{ChatMessage, OllamaThink},
    write_guard_registry,
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
    live_replay_target: Option<crate::services::reasoning_continuity::contract::ReplayTarget>,
    #[cfg(debug_assertions)] fixture_candidate: Option<
        crate::services::reasoning_continuity::contract::ReplayTarget,
    >,
    #[cfg(debug_assertions)] mut fixture_run: Option<
        &mut crate::services::reasoning_fixture_run::FixtureRunContext,
    >,
    mut journal: Option<&mut super::conversation_journal::ConversationJournal>,
) -> Result<super::agent_loop_finish::CompletedStreamTurn, String> {
    let (mut total_eval, mut total_prompt) = (Some(0), Some(0));
    let (mut last_prompt, mut last_eval) = (None, None);
    let mut generation = super::generation_metrics::GenerationAggregate::default();
    let mut breaker = circuit_breaker::CircuitBreaker::new();
    let write_guard_arc = write_guard_registry::lock(&session_id).await;
    let mut write_guard = write_guard_arc.lock().await;
    let mut plan_repairs = 0;
    #[cfg(debug_assertions)]
    let fixture_mode = fixture_run.is_some();
    #[cfg(not(debug_assertions))]
    let fixture_mode = false;
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
        agent_loop_support::ensure_not_cancelled(&cancel)?;
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
            live_replay_target: live_replay_target.as_ref(),
            #[cfg(debug_assertions)]
            fixture_candidate: fixture_candidate.as_ref(),
            enable_eager_tools: {
                #[cfg(debug_assertions)]
                {
                    !fixture_mode
                }
                #[cfg(not(debug_assertions))]
                {
                    true
                }
            },
        })
        .await?;
        generation.merge(request_output.generation);
        let eager_handle = request_output.eager_handle;
        let interrupted = request_output.interrupted;
        let plan_active = request_output.plan_active;
        let input_tokens = request_output.input_tokens;
        let result = request_output.result;
        if interrupted {
            eager_handle.abort();
            super::agent_loop_interrupted::handle(
                on_event,
                messages,
                &result,
                plan_active,
                input_tokens,
                configured_context,
                &compression,
                &mut last_prompt,
                &mut last_eval,
                cancel.clone(),
                journal.as_deref_mut(),
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
        let stop_after_tools =
            super::agent_loop_tool_turn::run(super::agent_loop_tool_turn::ToolTurnContext {
                on_event,
                messages,
                eager_handle: Some(eager_handle),
                result: &result,
                working_dir: &working_dir,
                permission_mode,
                session_id: &session_id,
                request_id: &request_id,
                cancel: cancel.clone(),
                write_guard: &mut write_guard,
                plan_active,
                fixture_mode,
                turn,
                model,
                breaker: &mut breaker,
                journal: journal.as_deref_mut(),
                tools: &mut tools,
                subagents: &mut subagents,
                compression: &compression,
                last_prompt: &mut last_prompt,
                last_eval: &mut last_eval,
                #[cfg(debug_assertions)]
                fixture_run: fixture_run.as_deref_mut(),
            })
            .await?;
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
