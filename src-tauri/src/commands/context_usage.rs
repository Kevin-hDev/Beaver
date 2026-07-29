use super::agent_chat_task::common;
use crate::services::agent_local::{
    model_size::{self, PromptTier},
    prompt_chat_compact, prompt_chat_detailed, prompt_compact, prompt_detailed, prompt_plan,
    tool_catalog,
};
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
    let behavior = if provider.as_deref() == Some("ollama") {
        crate::services::agent_local::ollama_behavior_overrides::get(&model)
    } else {
        None
    };

    let memory_usage =
        super::context_usage_memory::usage(provider.as_deref(), &model, &working_dir).await;
    let system_prompt_tokens = estimate(&base_prompt(
        &mode.mode,
        &model,
        &working_dir,
        &snap,
        &enabled_tool_names,
        behavior.as_deref(),
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
    model: &str,
    working_dir: &std::path::Path,
    snap: &crate::services::git_context::GitSnapshot,
    enabled_tool_names: &[String],
    behavior: Option<&str>,
) -> String {
    let prompt = match (mode == "chat", model_size::detect_tier(model)) {
        (true, PromptTier::Compact) => {
            prompt_chat_compact::build_with_behavior(working_dir, behavior)
        }
        (true, PromptTier::Detailed) => {
            prompt_chat_detailed::build_with_behavior(working_dir, behavior)
        }
        (false, PromptTier::Compact) => prompt_compact::build_with_behavior(
            working_dir,
            snap.is_git,
            snap.git_root.as_deref(),
            behavior,
        ),
        (false, PromptTier::Detailed) => prompt_detailed::build_with_behavior(
            working_dir,
            snap.is_git,
            snap.git_root.as_deref(),
            behavior,
        ),
    };
    crate::services::agent_local::tool_prompt_filter::filter_system_prompt(
        &prompt,
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
    if !response_language.is_empty() {
        total += estimate(&format!(
            "You MUST respond in {response_language}. All your answers, explanations and communications must be in {response_language}."
        ));
    }
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
    let listing = skills
        .iter()
        .map(|(name, desc)| format!("- {name}: {desc}"))
        .collect::<Vec<_>>()
        .join("\n");
    estimate(&format!(
        "## Available skills\nThe following skills are available. Use the `load_skill` tool to load one when relevant.\n{listing}"
    ))
}

fn estimate(input: &str) -> usize {
    crate::services::token_counting::estimate_text_tokens(input)
}
