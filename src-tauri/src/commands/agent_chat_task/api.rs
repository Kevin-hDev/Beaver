use super::common::{self, StreamMode};
use super::params::StreamTaskParams;
use crate::services::agent_local::tool_catalog;
use crate::services::agent_local::types_ollama::ChatMessage;
use crate::services::llm;
use crate::services::llm::{model_registry, tool_capable};

struct ApiCapabilities {
    tools: bool,
    thinking: bool,
    vision: bool,
}

pub(crate) async fn run(
    params: StreamTaskParams,
    mode: StreamMode,
    response_language: String,
) -> Result<Vec<ChatMessage>, String> {
    let canonical_provider = llm::route::canonical_provider_id(&params.provider);
    let ctx =
        crate::services::compress::context_resolve::resolve_api(canonical_provider, &params.model)
            .await;
    let caps = resolve_capabilities(&params, canonical_provider).await;
    let settings = crate::services::agent_local::agent_settings::load().await;
    let final_tools =
        super::api_tools::resolve(&params, &mode, caps.tools, &settings, canonical_provider);
    let extension_tools = if mode.is_chat {
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
    common::update_working_dir(&params.session_id, &working_dir).await;
    let plan_mode_active =
        resolve_plan_mode(&params).await && tool_catalog::has_plan_tools(&enabled_tool_names);

    let snap = common::collect_git_snapshot(&working_dir).await;
    let has_tools = !extension_tools.active().is_empty();
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
    super::api_images::sanitize_images(&params.on_event, &mut messages, caps.vision);
    if params.subagent_profile.is_some() {
        common::append_memory_context(&mut messages, memory_context);
        common::append_git_section(&mut messages, &snap);
    } else {
        let agent_md = common::agent_md_content(&mode, &working_dir).await;
        let skills = common::skills_tuples(
            !mode.is_chat
                && !mode.is_subagent
                && has_tools
                && tool_catalog::has_tool(&enabled_tool_names, "load_skill"),
        )
        .await;
        common::prepare_with_context(
            &mut messages,
            common::PromptContext {
                working_dir: &working_dir,
                snap: &snap,
                has_tools,
                agent_md_content: agent_md,
                skills: &skills,
                model: &params.model,
                mode: &mode.mode,
                response_language: &response_language,
                plan_mode_active,
                enabled_tool_names: &enabled_tool_names,
                behavior: None,
                memory_context,
            },
        );
    }
    if super::api_tools::todo_tools_enabled(&enabled_tool_names) {
        crate::services::agent_local::tool_todo::append_session_reminder(
            &mut messages,
            &params.session_id,
        )
        .await;
    }
    super::gemma4_thinking_guard::apply(&mut messages, canonical_provider, &params.model);

    let effective_reasoning_mode = crate::services::reasoning::normalize_for_model(
        canonical_provider,
        &params.model,
        params.reasoning_mode.as_deref(),
        caps.thinking,
    );
    let think_active =
        crate::services::reasoning::enabled(effective_reasoning_mode.as_deref(), params.think)
            && caps.thinking;
    llm::agent_loop::run_agent_loop(
        &params.on_event,
        &params.provider,
        &params.model,
        &mut messages,
        extension_tools,
        think_active,
        effective_reasoning_mode.as_deref(),
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

async fn resolve_capabilities(
    params: &StreamTaskParams,
    canonical_provider: &str,
) -> ApiCapabilities {
    let registry_caps = model_registry::lookup(canonical_provider, &params.model).await;
    let runtime_caps = llm::runtime_models::lookup(canonical_provider, &params.model);
    ApiCapabilities {
        tools: params.capability_hints.supports_tools.unwrap_or_else(|| {
            registry_caps
                .as_ref()
                .map(|c| c.supports_tools)
                .or_else(|| runtime_caps.as_ref().map(|model| model.supports_tools))
                .unwrap_or(false)
                || tool_capable::supports_tools(canonical_provider, &params.model)
        }),
        thinking: params
            .capability_hints
            .supports_thinking
            .unwrap_or_else(|| {
                params.provider == "codex-oauth"
                    || registry_caps
                        .as_ref()
                        .map(|c| c.supports_thinking)
                        .unwrap_or(false)
                    || runtime_caps
                        .as_ref()
                        .is_some_and(|model| model.supports_thinking)
                    || tool_capable::supports_thinking(canonical_provider, &params.model)
            }),
        vision: params.capability_hints.supports_vision.unwrap_or_else(|| {
            registry_caps
                .as_ref()
                .map(|c| c.supports_vision)
                .unwrap_or(false)
                || runtime_caps
                    .as_ref()
                    .is_some_and(|model| model.supports_vision)
                || params.provider == "codex-oauth"
                || tool_capable::supports_vision(canonical_provider, &params.model)
        }),
    }
}
