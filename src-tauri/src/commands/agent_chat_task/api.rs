use super::common::{self, StreamMode};
use super::params::StreamTaskParams;
use crate::services::agent_local::tool_catalog;
use crate::services::agent_local::types_ollama::ChatMessage;
use crate::services::llm;

pub(crate) async fn run(
    params: StreamTaskParams,
    mut messages: Vec<ChatMessage>,
    mode: StreamMode,
    response_language: String,
    journal: &mut Option<crate::services::agent_local::conversation_journal::ConversationJournal>,
) -> Result<crate::services::agent_local::agent_loop_finish::CompletedStreamTurn, String> {
    #[cfg(debug_assertions)]
    let mut params = params;
    let canonical_provider = llm::route::canonical_provider_id(&params.provider);
    let fast_mode =
        llm::fast_mode::for_session(&params.session_id, &params.provider, &params.model).await?;
    let ctx =
        crate::services::compress::context_resolve::resolve_api(canonical_provider, &params.model)
            .await;
    let caps =
        super::api_capabilities::resolve(&params.provider, &params.model, &params.capability_hints)
            .await;
    let settings = crate::services::agent_local::agent_settings::load().await;
    #[cfg(debug_assertions)]
    let fixture_mode = params.fixture_run.is_some();
    #[cfg(not(debug_assertions))]
    let fixture_mode = false;
    let final_tools = if fixture_mode {
        #[cfg(debug_assertions)]
        {
            params
                .fixture_run
                .as_ref()
                .map(|run| run.definitions().to_vec())
                .unwrap_or_default()
        }
        #[cfg(not(debug_assertions))]
        {
            Vec::new()
        }
    } else {
        super::api_tools::resolve(&params, &mode, caps.tools, &settings, canonical_provider)
    };
    let extension_tools = if fixture_mode || mode.is_chat {
        crate::services::agent_local::extension_tool_set::ExtensionToolSet::passthrough(final_tools)
    } else {
        crate::services::agent_local::extension_tool_set::ExtensionToolSet::prepare(
            final_tools,
            crate::services::agent_local::extension_tool_set::PrepareContext {
                session_id: &params.session_id,
                provider: canonical_provider,
                model: &params.model,
                context_window: ctx.configured,
                preserve_dynamic_tools: !params.tools.is_empty(),
            },
        )
        .await?
    };
    let enabled_tool_names = tool_catalog::tool_names(extension_tools.active());
    crate::services::agent_local::extension_tool_set::record_selection(
        &extension_tools,
        &params.session_id,
        &params.request_id,
        "extension_tools_selected",
    )
    .await;
    let working_dir = common::resolve_working_dir(&params.working_dir)?;
    common::update_working_dir(&params.session_id, &working_dir).await?;
    let plan_mode_active =
        resolve_plan_mode(&params).await && tool_catalog::has_plan_tools(&enabled_tool_names);

    let snap = common::collect_git_snapshot(&working_dir).await;
    let prompt_mode =
        crate::services::agent_local::system_prompt_defaults::mode_for_permission(&mode.mode);
    let prompt_tier =
        crate::services::agent_local::system_prompt_defaults::tier_for_model(&params.model);
    let beaver_prompt = crate::services::agent_local::system_prompt_defaults::beaver_prompt(
        prompt_mode,
        prompt_tier,
    );
    let prompt_settings = super::prompt_settings::load(&params.on_event);
    let instructions = crate::services::agent_local::system_prompt_resolver::resolve_global(
        &prompt_settings,
        prompt_mode,
        prompt_tier,
        &beaver_prompt,
    );
    let has_tools = !extension_tools.active().is_empty();
    let prepared_memory = crate::services::agent_local::memory_context::prepare(
        &params.session_id,
        &messages,
        &working_dir,
        ctx.configured,
        params.subagent_profile.is_some(),
    )
    .await;
    let _memory_context_tokens = prepared_memory.tokens;
    let memory_context = prepared_memory.section;
    let _memory_guard = prepared_memory.guard;
    super::api_images::sanitize_images(&params.on_event, &mut messages, caps.vision);
    let context_usage_seed = if params.subagent_profile.is_some() {
        let seed = super::context_usage_seed::for_subagent(memory_context.as_deref(), &snap);
        common::append_memory_context(&mut messages, memory_context);
        common::append_git_section(&mut messages, &snap);
        seed
    } else {
        let agent_md = common::agent_md_content(&mode, &working_dir).await;
        let skills = common::skills_tuples(
            !mode.is_chat
                && !mode.is_subagent
                && has_tools
                && tool_catalog::has_tool(&enabled_tool_names, "load_skill"),
        )
        .await;
        let prompt_context = common::PromptContext {
            working_dir: &working_dir,
            outputs_dir: params.outputs_dir.as_deref(),
            snap: &snap,
            has_tools,
            agent_md_content: agent_md,
            skills: &skills,
            model: &params.model,
            mode: &mode.mode,
            response_language: &response_language,
            plan_mode_active,
            enabled_tool_names: &enabled_tool_names,
            instructions: &instructions,
            memory_context,
        };
        let seed = super::context_usage_seed::for_prompt(&prompt_context);
        common::prepare_with_context(&mut messages, prompt_context);
        seed
    };
    if super::api_tools::todo_tools_enabled(&enabled_tool_names) {
        crate::services::agent_local::tool_todo::append_session_reminder(
            &mut messages,
            &params.session_id,
        )
        .await;
    }
    super::gemma4_thinking_guard::apply(&mut messages, canonical_provider, &params.model);

    let (think_active, effective_reasoning_mode) = match params.reasoning_profile.as_ref() {
        Some(profile) => (profile.active, profile.mode_name.clone()),
        None => {
            let mode = crate::services::reasoning::normalize_for_model(
                canonical_provider,
                &params.model,
                params.reasoning_mode.as_deref(),
                caps.thinking,
            );
            (
                crate::services::reasoning::enabled(mode.as_deref(), params.think) && caps.thinking,
                mode,
            )
        }
    };
    #[cfg(debug_assertions)]
    let mut fixture_run = params.fixture_run.take();
    let completed = llm::agent_loop::run_agent_loop(
        &params.on_event,
        &params.provider,
        fast_mode,
        &params.model,
        &mut messages,
        extension_tools,
        think_active,
        effective_reasoning_mode.as_deref(),
        working_dir,
        params.session_id.clone(),
        params.request_id.clone(),
        params.parent_message_inbox.clone(),
        params.cancel.clone(),
        ctx.native,
        ctx.configured,
        &mode.mode,
        plan_mode_active,
        context_usage_seed,
        params.continuation_target.clone(),
        #[cfg(debug_assertions)]
        fixture_run.as_mut(),
        journal.as_mut(),
    )
    .await?;
    finish_turn(&params, journal, completed, messages).await
}

pub(crate) async fn finish_turn(
    params: &StreamTaskParams,
    journal: &mut Option<crate::services::agent_local::conversation_journal::ConversationJournal>,
    completed: crate::services::agent_local::agent_loop_finish::CompletedStreamTurn,
    messages: Vec<ChatMessage>,
) -> Result<crate::services::agent_local::agent_loop_finish::CompletedStreamTurn, String> {
    if let Some(journal) = journal.as_mut() {
        journal.commit_turn().await?;
        let (turn_id, user_message_id, assistant_message_id) = journal.turn_ids();
        super::reasoning_diagnostics::record_persisted(
            &params.session_id,
            &params.request_id,
            turn_id,
            assistant_message_id,
        )
        .await;
        let _ = params.on_event.send(
            crate::services::agent_local::types_ollama::StreamEvent::TurnCommitted {
                turn_id: turn_id.to_string(),
                user_message_id: user_message_id.to_string(),
                assistant_message_id: assistant_message_id.to_string(),
            },
        );
    }
    crate::services::agent_local::stream_diagnostics::record_completed(
        &params.session_id,
        &params.request_id,
    )
    .await;
    Ok(completed.with_messages(messages))
}

async fn resolve_plan_mode(params: &StreamTaskParams) -> bool {
    match params.plan_mode {
        Some(value) => value,
        None => crate::services::agent_local::tool_plan::is_enabled(&params.session_id).await,
    }
}
