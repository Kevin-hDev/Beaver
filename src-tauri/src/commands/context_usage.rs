use super::agent_chat_task::common;
use crate::services::agent_local::{
    model_size::{self, PromptTier},
    prompt_chat_compact, prompt_chat_detailed, prompt_compact, prompt_detailed, prompt_plan,
    tool_catalog, tool_definitions_chat, tool_definitions_mcp, tool_dispatcher,
};
use serde::Serialize;
use serde_json::Value;

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
    let working_dir = common::resolve_working_dir(&working_dir)?;
    let snap = common::collect_git_snapshot(&working_dir).await;
    let has_tools =
        mode.is_chat || provider.as_deref() == Some("ollama") || supports_tools.unwrap_or(false);
    let settings = crate::services::agent_local::agent_settings::load().await;
    let defs = filtered_tool_definitions(&mode.mode, has_tools, &settings.enabled_optional_tools);
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
    let (system_tool_definition_tokens, mcp_definition_tokens) = tool_definition_tokens(defs);

    Ok(HiddenContextUsage {
        system_prompt_tokens,
        meta_context_tokens,
        skill_context_tokens,
        memory_context_tokens,
        system_tool_definition_tokens,
        mcp_definition_tokens,
    })
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

fn filtered_tool_definitions(
    mode: &str,
    has_tools: bool,
    enabled_optional_tools: &[String],
) -> Vec<Value> {
    if !has_tools {
        return vec![];
    }
    let defs = if mode == "chat" {
        tool_definitions_chat::get_chat_tool_definitions()
    } else {
        tool_dispatcher::get_tool_definitions()
    };
    tool_catalog::filter_tool_definitions(defs, enabled_optional_tools)
}

fn tool_definition_tokens(defs: Vec<Value>) -> (usize, usize) {
    let mcp_names = mcp_tool_names();
    defs.into_iter().fold((0, 0), |(system, mcp), def| {
        let tokens = estimate(&def.to_string());
        if tool_name(&def).is_some_and(|name| mcp_names.contains(&name)) {
            (system, mcp + tokens)
        } else {
            (system + tokens, mcp)
        }
    })
}

fn mcp_tool_names() -> Vec<String> {
    tool_definitions_mcp::mcp_tool_definitions()
        .iter()
        .filter_map(tool_name)
        .collect()
}

fn tool_name(def: &Value) -> Option<String> {
    def.get("function")?
        .get("name")?
        .as_str()
        .map(ToString::to_string)
}

fn estimate(input: &str) -> usize {
    crate::services::token_counting::estimate_text_tokens(input)
}
