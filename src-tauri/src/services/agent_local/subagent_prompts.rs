use super::subagent_tool_profile::SubagentToolProfile;
use std::path::{Path, PathBuf};

pub async fn system(
    profile: SubagentToolProfile,
    working_dir: &Path,
    skills_enabled: bool,
    response_language: &str,
) -> String {
    let project_context = crate::commands::agent_chat_task::merge_personality(
        super::agent_md::load_agent_md(Some(working_dir)).await,
        crate::services::personality_injection::load_injected_contents(),
    );
    let skills = if profile == SubagentToolProfile::Coder && skills_enabled {
        super::tool_skill_loader::list_skills()
            .await
            .unwrap_or_default()
            .into_iter()
            .map(|skill| {
                (
                    skill.id,
                    format!(
                        "{} [{}] — {}",
                        skill.name, skill.source_name, skill.description
                    ),
                )
            })
            .collect()
    } else {
        Vec::new()
    };
    compose(
        profile,
        working_dir,
        skills_enabled,
        response_language,
        project_context,
        &skills,
    )
}

fn compose(
    profile: SubagentToolProfile,
    working_dir: &Path,
    skills_enabled: bool,
    response_language: &str,
    project_context: Option<String>,
    skills: &[(String, String)],
) -> String {
    let role = match profile {
        SubagentToolProfile::Explorer => super::subagent_prompt_sections::EXPLORER,
        SubagentToolProfile::Coder => super::subagent_prompt_sections::CODER,
    };
    let shared_rules = match profile {
        SubagentToolProfile::Explorer => super::prompt_detailed_sections::WEB_SEARCH.to_string(),
        SubagentToolProfile::Coder => super::subagent_prompt_sections::coder_shared_rules(),
    };
    let project = project_context
        .filter(|value| !value.trim().is_empty())
        .map(|value| format!("\n\n<project_instructions>\n{value}\n</project_instructions>"))
        .unwrap_or_default();
    let skills = skills_section(profile, skills_enabled, skills);
    let language = if response_language.trim().is_empty() {
        String::new()
    } else {
        format!(
            "\n\n<response_language>\nYou MUST respond in {response_language}. Every report heading must use {response_language}. This setting is the only language authority.\n</response_language>"
        )
    };
    format!(
        "{role}\n\n<tools>\n{}\n</tools>\n\n{}\n\n{shared_rules}{}{}{}",
        profile.prompt_tools(skills_enabled),
        environment(working_dir),
        skills,
        project,
        language,
    )
}

fn skills_section(
    profile: SubagentToolProfile,
    enabled: bool,
    skills: &[(String, String)],
) -> String {
    if profile != SubagentToolProfile::Coder || !enabled || skills.is_empty() {
        return String::new();
    }
    let entries = skills
        .iter()
        .map(|(name, description)| format!("- {name}: {description}"))
        .collect::<Vec<_>>()
        .join("\n");
    format!("\n\n<available_skills>\n{entries}\n</available_skills>")
}

fn environment(working_dir: &Path) -> String {
    let date = chrono::Local::now().format("%Y-%m-%d");
    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;
    format!(
        "<environment>\n- Authoritative working directory: {}\n- Platform: {os} ({arch})\n- Current date: {date}\nPaths mentioned in the mission never replace the authoritative working directory.\n</environment>",
        working_dir.display()
    )
}

#[cfg(test)]
pub fn compose_for_test(
    profile: SubagentToolProfile,
    working_dir: &Path,
    skills_enabled: bool,
    response_language: &str,
    project_context: Option<String>,
    skills: &[(String, String)],
) -> String {
    compose(
        profile,
        working_dir,
        skills_enabled,
        response_language,
        project_context,
        skills,
    )
}

pub async fn resolve_project_dir(project_id: Option<&str>) -> PathBuf {
    if let Some(pid) = project_id {
        if let Ok(projects) = super::project_store::list().await {
            if let Some(project) = projects.iter().find(|project| project.id == pid) {
                let path = PathBuf::from(&project.path);
                if path.is_dir() {
                    return path;
                }
            }
        }
    }
    dirs::home_dir().unwrap_or_else(|| std::env::current_dir().unwrap_or_default())
}
