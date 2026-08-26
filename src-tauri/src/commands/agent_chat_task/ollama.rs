use super::common::{self, StreamMode};
use super::params::StreamTaskParams;
use crate::services::agent_local::agent_loop;
use crate::services::agent_local::agent_settings::AgentSettings;
use crate::services::agent_local::tool_catalog;
use crate::services::agent_local::tool_dispatcher;
use crate::services::agent_local::types_ollama::{ChatMessage, StreamEvent};

pub(crate) async fn run(
    params: StreamTaskParams,
    mut messages: Vec<ChatMessage>,
    mode: StreamMode,
    response_language: String,
    journal: &mut Option<crate::services::agent_local::conversation_journal::ConversationJournal>,
) -> Result<crate::services::agent_local::agent_loop_finish::CompletedStreamTurn, String> {
    let ctx = crate::services::compress::context_resolve::resolve_ollama(&params.model).await;
    let settings = crate::services::agent_local::agent_settings::load().await;
    let final_tools = resolve_tools(&params, &mode, &settings);
    let extension_tools = if mode.is_chat {
        crate::services::agent_local::extension_tool_set::ExtensionToolSet::passthrough(final_tools)
    } else {
        crate::services::agent_local::extension_tool_set::ExtensionToolSet::prepare(
            final_tools,
            crate::services::agent_local::extension_tool_set::PrepareContext {
                session_id: &params.session_id,
                provider: "ollama",
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
    let ollama_think = super::ollama_thinking::resolve(&params).await?;
    let prompt_mode =
        crate::services::agent_local::system_prompt_defaults::mode_for_permission(&mode.mode);
    let prompt_tier = ctx.prompt_tier.unwrap_or_else(|| {
        crate::services::agent_local::system_prompt_defaults::tier_for_model(&params.model)
    });
    let beaver_prompt = crate::services::agent_local::system_prompt_defaults::beaver_prompt(
        prompt_mode,
        prompt_tier,
    );
    let prompt_settings = super::prompt_settings::load(&params.on_event);
    let instructions =
        match crate::services::agent_local::system_prompt_resolver::resolve_ollama_without_native(
            &prompt_settings,
            &params.model,
            prompt_mode,
            prompt_tier,
            &beaver_prompt,
        ) {
            Some(view) => view,
            None => {
                let client =
                    crate::services::agent_local::ollama_client::OllamaClient::from_global()?;
                let native_prompt = crate::services::agent_local::ollama_native_prompts::get(
                    &client,
                    &params.model,
                )
                .await;
                crate::services::agent_local::system_prompt_resolver::resolve_ollama_native(
                    &prompt_settings,
                    &params.model,
                    prompt_mode,
                    prompt_tier,
                    &native_prompt,
                    &beaver_prompt,
                )
            }
        };
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
    let image_report = crate::services::llm::vision::sanitize_messages(&mut messages, true);
    if image_report.invalid_removed > 0 {
        let _ = params.on_event.send(StreamEvent::Notice {
            message_key: crate::services::llm::vision::NOTICE_IMAGE_SKIPPED.to_string(),
        });
    }

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
                && tool_catalog::has_tool(&enabled_tool_names, "load_skill"),
        )
        .await;
        let prompt_context = common::PromptContext {
            working_dir: &working_dir,
            outputs_dir: params.outputs_dir.as_deref(),
            snap: &snap,
            has_tools: true,
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
    if todo_tools_enabled(&enabled_tool_names) {
        crate::services::agent_local::tool_todo::append_session_reminder(
            &mut messages,
            &params.session_id,
        )
        .await;
    }

    let completed = agent_loop::run_agent_loop(
        &params.on_event,
        &mut messages,
        &params.model,
        extension_tools,
        ollama_think,
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
        journal.as_mut(),
    )
    .await?;
    super::api::finish_turn(&params, journal, completed, messages).await
}

async fn resolve_plan_mode(params: &StreamTaskParams) -> bool {
    match params.plan_mode {
        Some(value) => value,
        None => crate::services::agent_local::tool_plan::is_enabled(&params.session_id).await,
    }
}

fn resolve_tools(
    params: &StreamTaskParams,
    mode: &StreamMode,
    settings: &AgentSettings,
) -> Vec<serde_json::Value> {
    let defs = definitions_for_mode(mode.is_chat, &params.tools);
    tool_catalog::filter_tool_definitions(defs, &settings.enabled_optional_tools)
}

fn definitions_for_mode(is_chat: bool, requested: &[serde_json::Value]) -> Vec<serde_json::Value> {
    if is_chat {
        tool_dispatcher::get_chat_tool_definitions()
    } else if !requested.is_empty() {
        requested.to_vec()
    } else {
        tool_dispatcher::get_tool_definitions()
    }
}

fn todo_tools_enabled(enabled_tool_names: &[String]) -> bool {
    tool_catalog::has_any_tool(
        enabled_tool_names,
        &[
            "todo_write",
            "todo_history",
            "todo_pause",
            "todo_resume",
            "todo_delete",
        ],
    )
}

#[cfg(test)]
use super::ollama_thinking::canonical as canonical_ollama_think;

#[cfg(test)]
#[path = "ollama_tests.rs"]
mod tests;
