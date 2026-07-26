use super::common::{self, StreamMode};
use super::params::StreamTaskParams;
use crate::services::agent_local::agent_loop;
use crate::services::agent_local::agent_settings::AgentSettings;
use crate::services::agent_local::tool_catalog;
use crate::services::agent_local::tool_dispatcher;
use crate::services::agent_local::types_ollama::{ChatMessage, OllamaThink, StreamEvent};

pub(crate) async fn run(
    params: StreamTaskParams,
    mode: StreamMode,
    response_language: String,
) -> Result<Vec<ChatMessage>, String> {
    let ctx = crate::services::compress::context_resolve::resolve_ollama(&params.model).await;
    let settings = crate::services::agent_local::agent_settings::load().await;
    let final_tools = resolve_tools(&params, &mode, &settings);
    let enabled_tool_names = tool_catalog::tool_names(&final_tools);
    let working_dir = common::resolve_working_dir(&params.working_dir)?;
    common::update_working_dir(&params.session_id, &working_dir).await;
    let plan_mode_active =
        resolve_plan_mode(&params).await && tool_catalog::has_plan_tools(&enabled_tool_names);

    let snap = common::collect_git_snapshot(&working_dir).await;
    let ollama_think = resolve_ollama_think(&params);
    let behavior = crate::services::agent_local::ollama_behavior_overrides::get(&params.model);
    let mut messages = params.messages;
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

    if params.subagent_profile.is_some() {
        common::append_memory_context(&mut messages, memory_context);
        common::append_git_section(&mut messages, &snap);
    } else {
        let agent_md = common::agent_md_content(&mode, &working_dir).await;
        let skills = common::skills_tuples(
            !mode.is_chat
                && !mode.is_subagent
                && tool_catalog::has_tool(&enabled_tool_names, "load_skill"),
        )
        .await;
        common::prepare_with_context(
            &mut messages,
            common::PromptContext {
                working_dir: &working_dir,
                snap: &snap,
                has_tools: true,
                agent_md_content: agent_md,
                skills: &skills,
                model: &params.model,
                mode: &mode.mode,
                response_language: &response_language,
                plan_mode_active,
                enabled_tool_names: &enabled_tool_names,
                behavior: behavior.as_deref(),
                memory_context,
            },
        );
    }
    if todo_tools_enabled(&enabled_tool_names) {
        crate::services::agent_local::tool_todo::append_session_reminder(
            &mut messages,
            &params.session_id,
        )
        .await;
    }

    agent_loop::run_agent_loop(
        &params.on_event,
        &mut messages,
        &params.model,
        final_tools,
        ollama_think,
        working_dir,
        params.session_id.clone(),
        params.request_id.clone(),
        params.parent_message_inbox.clone(),
        params.cancel,
        ctx.native,
        ctx.configured,
        &mode.mode,
        plan_mode_active,
    )
    .await?;
    Ok(messages)
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
    let defs = if !params.tools.is_empty() {
        params.tools.clone()
    } else if mode.is_chat {
        tool_dispatcher::get_chat_tool_definitions()
    } else {
        tool_dispatcher::get_tool_definitions()
    };
    tool_catalog::filter_tool_definitions(defs, &settings.enabled_optional_tools)
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
            "agent_diagnostics",
        ],
    )
}

fn resolve_ollama_think(params: &StreamTaskParams) -> OllamaThink {
    let supports_thinking = params
        .capability_hints
        .supports_thinking
        .unwrap_or_else(|| {
            crate::services::reasoning::provider_model_supports_thinking("ollama", &params.model)
        });
    let effective_mode = crate::services::reasoning::normalize_for_model(
        "ollama",
        &params.model,
        params.reasoning_mode.as_deref(),
        supports_thinking,
    );
    crate::services::reasoning::ollama_think(
        &params.model,
        effective_mode.as_deref(),
        params.think && supports_thinking,
    )
    .unwrap_or(OllamaThink::Bool(false))
}
