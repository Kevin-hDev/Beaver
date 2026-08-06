use super::agent_chat_task::common;
use crate::services::agent_local::{prompt_plan, tool_catalog};
use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HiddenContextUsage {
    pub system_prompt_tokens: usize,
    pub meta_context_tokens: usize,
    pub skill_context_tokens: usize,
    pub memory_context_tokens: usize,
    pub system_tool_definition_tokens: usize,
    pub mcp_definition_tokens: usize,
}

#[tauri::command]
pub async fn estimate_context_hidden_usage(
    session_id: String,
    model: String,
    provider: Option<String>,
    working_dir: Option<String>,
    permission_mode: Option<String>,
    plan_mode: Option<bool>,
    supports_tools: Option<bool>,
) -> Result<HiddenContextUsage, String> {
    let mode = common::resolve_permission_mode(permission_mode.as_deref()).await;
    let Some(working_dir) =
        super::agent_working_dir::resolve_existing_for_session(&session_id, working_dir.as_deref())
            .await?
            .map(|resolved| resolved.path)
    else {
        return Ok(empty_usage());
    };
    let snap = common::collect_git_snapshot(&working_dir).await;
    let has_tools =
        mode.is_chat || provider.as_deref() == Some("ollama") || supports_tools.unwrap_or(false);
    let settings = crate::services::agent_local::agent_settings::load().await;
    let defs = super::context_usage_tools::filtered_definitions(
        &mode.mode,
        has_tools,
        &settings.enabled_optional_tools,
    );
    let defs = provider
        .as_deref()
        .filter(|provider_id| *provider_id != "ollama")
        .map(|provider_id| {
            let canonical = crate::services::llm::route::canonical_provider_id(provider_id);
            super::agent_chat_task::tool_policy::apply(canonical, &model, defs.clone()).tools
        })
        .unwrap_or(defs);
    let enabled_tool_names = tool_catalog::tool_names(&defs);
    let plan_active = match plan_mode {
        Some(value) => value,
        None => crate::services::agent_local::tool_plan::is_enabled(&session_id).await,
    } && tool_catalog::has_plan_tools(&enabled_tool_names);
    let prompt_mode =
        crate::services::agent_local::system_prompt_defaults::mode_for_permission(&mode.mode);
    let prompt_tier = crate::services::agent_local::system_prompt_defaults::tier_for_model(&model);
    let beaver_prompt = crate::services::agent_local::system_prompt_defaults::beaver_prompt(
        prompt_mode,
        prompt_tier,
    );
    let prompt_settings =
        crate::services::agent_local::system_prompt_store::snapshot_for_runtime().settings;
    let instructions = if provider.as_deref() == Some("ollama") {
        match crate::services::agent_local::system_prompt_resolver::resolve_ollama_without_native(
            &prompt_settings,
            &model,
            prompt_mode,
            prompt_tier,
            &beaver_prompt,
        ) {
            Some(view) => view,
            None => {
                let client = crate::services::agent_local::ollama_client::OllamaClient::new();
                let native_prompt =
                    crate::services::agent_local::ollama_native_prompts::get(&client, &model).await;
                crate::services::agent_local::system_prompt_resolver::resolve_ollama_native(
                    &prompt_settings,
                    &model,
                    prompt_mode,
                    prompt_tier,
                    &native_prompt,
                    &beaver_prompt,
                )
            }
        }
    } else {
        crate::services::agent_local::system_prompt_resolver::resolve_global(
            &prompt_settings,
            prompt_mode,
            prompt_tier,
            &beaver_prompt,
        )
    };

    let memory_usage =
        super::context_usage_memory::usage(provider.as_deref(), &model, &working_dir).await;
    let system_prompt_tokens = estimate(&base_prompt(
        &mode.mode,
        &working_dir,
        &snap,
        &enabled_tool_names,
        &instructions,
    ))
    .saturating_add(memory_usage.prompt_tokens);
    let meta_context_tokens = meta_context_tokens(&mode, &working_dir, &snap, plan_active).await;
    let skill_context_tokens = skill_context_tokens(
        &mode,
        !defs.is_empty() && tool_catalog::has_tool(&enabled_tool_names, "load_skill"),
    )
    .await;
    let memory_context_tokens = memory_usage.summary_tokens;
    let (system_tool_definition_tokens, mcp_definition_tokens) =
        super::context_usage_tools::definition_tokens(defs);

    Ok(HiddenContextUsage {
        system_prompt_tokens,
        meta_context_tokens,
        skill_context_tokens,
        memory_context_tokens,
        system_tool_definition_tokens,
        mcp_definition_tokens,
    })
}

fn empty_usage() -> HiddenContextUsage {
    HiddenContextUsage {
        system_prompt_tokens: 0,
        meta_context_tokens: 0,
        skill_context_tokens: 0,
        memory_context_tokens: 0,
        system_tool_definition_tokens: 0,
        mcp_definition_tokens: 0,
    }
}

fn base_prompt(
    mode: &str,
    working_dir: &std::path::Path,
    snap: &crate::services::git_context::GitSnapshot,
    enabled_tool_names: &[String],
    instructions: &crate::services::agent_local::system_prompt_types::SystemPromptView,
) -> String {
    crate::services::agent_local::chat_prompts::compose_instructions_with_runtime(
        mode,
        working_dir,
        snap.is_git,
        snap.git_root.as_deref(),
        instructions,
        enabled_tool_names,
    )
}

async fn meta_context_tokens(
    mode: &common::StreamMode,
    working_dir: &std::path::Path,
    snap: &crate::services::git_context::GitSnapshot,
    plan_active: bool,
) -> usize {
    let mut total = 0;
    if let Some(agent_md) = common::agent_md_content(mode, working_dir).await {
        total += estimate(&agent_md);
    }
    if let Some(git_section) = crate::services::git_context::format_git_section(snap) {
        total += estimate(&git_section);
    }
    let response_language = common::response_language();
    total += crate::services::agent_local::chat_prompt_sections::response_language_instruction(
        &response_language,
    )
    .as_deref()
    .map(estimate)
    .unwrap_or(0);
    if plan_active {
        total += estimate(&prompt_plan::plan_mode_prompt());
    }
    total
}

async fn skill_context_tokens(mode: &common::StreamMode, has_tools: bool) -> usize {
    let skills = common::skills_tuples(!mode.is_chat && !mode.is_subagent && has_tools).await;
    if skills.is_empty() {
        return 0;
    }
    crate::services::agent_local::chat_prompt_sections::skills_listing_section(&skills)
        .as_deref()
        .map(estimate)
        .unwrap_or(0)
}

fn estimate(input: &str) -> usize {
    crate::services::token_counting::estimate_text_tokens(input)
}
