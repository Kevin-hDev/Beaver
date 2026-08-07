use super::workspace_prompt::append_outputs_directory;
use crate::services::agent_local::agent_md;
use crate::services::agent_local::agent_settings;
use crate::services::agent_local::chat_prompts::prepare_messages_with_tools;
use crate::services::agent_local::session_store;
use crate::services::agent_local::tool_skill_loader;
use crate::services::agent_local::types_ollama::ChatMessage;
use crate::services::git_context::{self, GitSnapshot};
use crate::services::personality_injection;
use std::path::{Path, PathBuf};

pub(crate) struct StreamMode {
    pub mode: String,
    pub is_chat: bool,
    pub is_subagent: bool,
}

pub(crate) struct PromptContext<'a> {
    pub working_dir: &'a Path,
    pub outputs_dir: Option<&'a Path>,
    pub snap: &'a GitSnapshot,
    pub has_tools: bool,
    pub agent_md_content: Option<String>,
    pub skills: &'a [(String, String)],
    pub model: &'a str,
    pub mode: &'a str,
    pub response_language: &'a str,
    pub plan_mode_active: bool,
    pub enabled_tool_names: &'a [String],
    pub instructions: &'a crate::services::agent_local::system_prompt_types::SystemPromptView,
    pub memory_context: Option<String>,
}

pub(crate) async fn resolve_permission_mode(override_mode: Option<&str>) -> StreamMode {
    let stored = agent_settings::get_permission_mode().await;
    let mode = match override_mode {
        Some(m) if matches!(m, "auto" | "manual" | "chat" | "subagent") => {
            if is_more_permissive(m, &stored) {
                stored
            } else {
                m.to_string()
            }
        }
        _ => stored,
    };
    StreamMode {
        is_chat: mode == "chat",
        is_subagent: mode == "subagent",
        mode,
    }
}

pub(crate) fn response_language() -> String {
    crate::services::config::read_config()
        .map(|c| c.advanced.response_language)
        .unwrap_or_default()
}

pub(crate) fn resolve_working_dir(working_dir: &Path) -> Result<PathBuf, String> {
    if working_dir.is_dir() {
        return working_dir.canonicalize().map_err(|err| {
            ::log::warn!("[agent] canonicalize dir: {err}");
            "Répertoire inaccessible".to_string()
        });
    }
    Err("Répertoire de travail introuvable".to_string())
}

pub(crate) async fn update_working_dir(session_id: &str, working_dir: &Path) -> Result<(), String> {
    session_store::refresh_working_dir(session_id, &working_dir.to_string_lossy())
        .await
        .map_err(|_| "Impossible d'enregistrer le dossier de travail.".to_string())
}

pub(crate) async fn collect_git_snapshot(working_dir: &Path) -> GitSnapshot {
    let wd = working_dir.to_path_buf();
    tokio::time::timeout(
        std::time::Duration::from_secs(3),
        tokio::task::spawn_blocking(move || git_context::detect_git(&wd)),
    )
    .await
    .ok()
    .and_then(|r| r.ok())
    .unwrap_or_default()
}

pub(crate) fn append_git_section(messages: &mut [ChatMessage], snap: &GitSnapshot) {
    if let Some(section) = git_context::format_git_section(snap) {
        if let Some(first) = messages.first_mut().filter(|m| m.role == "system") {
            first.content.push_str(&format!("\n\n{section}"));
        }
    }
}

pub(crate) async fn agent_md_content(mode: &StreamMode, working_dir: &Path) -> Option<String> {
    if mode.is_chat || mode.is_subagent {
        return None;
    }
    merge_personality(
        agent_md::load_agent_md(Some(working_dir)).await,
        personality_injection::load_injected_contents(),
    )
}

pub(crate) async fn skills_tuples(load: bool) -> Vec<(String, String)> {
    if !load {
        return vec![];
    }
    tool_skill_loader::list_skills()
        .await
        .unwrap_or_default()
        .iter()
        .map(|skill| {
            (
                skill.id.clone(),
                format!(
                    "{} [{}] — {}",
                    skill.name, skill.source_name, skill.description
                ),
            )
        })
        .collect()
}

pub(crate) fn prepare_with_context(messages: &mut Vec<ChatMessage>, ctx: PromptContext<'_>) {
    let plan_mode_active = ctx.plan_mode_active;
    prepare_messages_with_tools(
        messages,
        ctx.working_dir,
        ctx.snap.is_git,
        ctx.snap.git_root.as_deref(),
        ctx.has_tools,
        ctx.agent_md_content,
        ctx.skills,
        ctx.model,
        ctx.mode,
        ctx.response_language,
        ctx.enabled_tool_names,
        ctx.instructions,
    );
    append_memory_context(messages, ctx.memory_context);
    append_git_section(messages, ctx.snap);
    append_outputs_directory(messages, ctx.outputs_dir);
    if plan_mode_active {
        append_plan_mode(messages);
    }
}

pub(crate) fn append_memory_context(messages: &mut [ChatMessage], memory_context: Option<String>) {
    let Some(memory_context) = memory_context.filter(|content| !content.is_empty()) else {
        return;
    };
    if let Some(first) = messages
        .first_mut()
        .filter(|message| message.role == "system")
    {
        first.content.push_str("\n\n");
        first.content.push_str(&memory_context);
    }
}

fn append_plan_mode(messages: &mut [ChatMessage]) {
    if let Some(first) = messages.first_mut().filter(|m| m.role == "system") {
        first.content.push_str("\n\n");
        first
            .content
            .push_str(&crate::services::agent_local::prompt_plan::plan_mode_prompt());
    }
}

pub(crate) fn merge_personality(
    agent_md: Option<String>,
    personality: Option<String>,
) -> Option<String> {
    match (agent_md, personality) {
        (Some(a), Some(p)) => Some(format!("{a}\n\n{p}")),
        (Some(a), None) => Some(a),
        (None, Some(p)) => Some(p),
        (None, None) => None,
    }
}

fn permission_level(mode: &str) -> u8 {
    match mode {
        "manual" => 0,
        "chat" => 1,
        "subagent" => 2,
        "auto" => 3,
        _ => 0,
    }
}

fn is_more_permissive(requested: &str, stored: &str) -> bool {
    permission_level(requested) > permission_level(stored)
}
