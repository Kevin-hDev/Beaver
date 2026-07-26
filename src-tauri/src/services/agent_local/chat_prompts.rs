use crate::services::agent_local::model_size::{self, PromptTier};
use crate::services::agent_local::types_ollama::ChatMessage;
use crate::services::agent_local::{
    prompt_chat_compact, prompt_chat_detailed, prompt_compact, prompt_detailed,
};
use std::path::Path;

fn build_system_message(content: String) -> ChatMessage {
    ChatMessage {
        role: "system".to_string(),
        content,
        images: None,
        tool_calls: None,
        tool_name: None,
        tool_call_id: None,
        reasoning_content: None,
    }
}

fn prepend_tool_system_prompt(
    messages: &mut Vec<ChatMessage>,
    working_dir: &Path,
    is_git: bool,
    git_root: Option<&Path>,
    model: &str,
    behavior: Option<&str>,
) {
    if messages.first().is_some_and(|m| m.role == "system") {
        return;
    }
    let tier = model_size::detect_tier(model);
    let prompt = match tier {
        PromptTier::Compact => {
            prompt_compact::build_with_behavior(working_dir, is_git, git_root, behavior)
        }
        PromptTier::Detailed => {
            prompt_detailed::build_with_behavior(working_dir, is_git, git_root, behavior)
        }
    };
    messages.insert(0, build_system_message(prompt));
}

fn filter_tool_prompt(messages: &mut [ChatMessage], enabled_tool_names: &[String]) {
    if let Some(first) = messages.first_mut().filter(|m| m.role == "system") {
        first.content =
            super::tool_prompt_filter::filter_system_prompt(&first.content, enabled_tool_names);
    }
}

pub fn prepend_agent_md_context(messages: &mut Vec<ChatMessage>, agent_md: Option<String>) {
    let content = match agent_md {
        Some(c) if !c.is_empty() => c,
        _ => return,
    };
    if let Some(first) = messages.first_mut().filter(|m| m.role == "system") {
        first.content = format!("{content}\n\n{}", first.content);
    } else {
        messages.insert(0, build_system_message(content));
    }
}

#[cfg(test)]
pub fn prepare_messages(
    messages: &mut Vec<ChatMessage>,
    working_dir: &Path,
    is_git: bool,
    git_root: Option<&Path>,
    has_tools: bool,
    agent_md: Option<String>,
    skills: &[(String, String)],
    model: &str,
    mode: &str,
    response_language: &str,
) {
    let enabled_tool_names = default_prompt_tool_names();
    prepare_messages_with_tools(
        messages,
        working_dir,
        is_git,
        git_root,
        has_tools,
        agent_md,
        skills,
        model,
        mode,
        response_language,
        &enabled_tool_names,
        None,
    );
}

pub fn prepare_messages_with_tools(
    messages: &mut Vec<ChatMessage>,
    working_dir: &Path,
    is_git: bool,
    git_root: Option<&Path>,
    has_tools: bool,
    agent_md: Option<String>,
    skills: &[(String, String)],
    model: &str,
    mode: &str,
    response_language: &str,
    enabled_tool_names: &[String],
    behavior: Option<&str>,
) {
    if mode == "chat" {
        prepend_chat_system_prompt(messages, working_dir, model, behavior);
        filter_tool_prompt(messages, enabled_tool_names);
        prepend_web_search_status(messages);
    } else {
        prepend_tool_system_prompt(messages, working_dir, is_git, git_root, model, behavior);
        filter_tool_prompt(messages, enabled_tool_names);
        prepend_web_search_status(messages);
        super::disabled_tools_hint::prepend(messages, enabled_tool_names);
        if has_tools && !skills.is_empty() {
            prepend_skills_listing(messages, skills);
        }
        prepend_agent_md_context(messages, agent_md);
    }
    append_response_language(messages, response_language);
}

#[cfg(test)]
fn default_prompt_tool_names() -> Vec<String> {
    super::tool_catalog::catalog()
        .iter()
        .map(|tool| tool.id.to_string())
        .collect()
}

fn prepend_chat_system_prompt(
    messages: &mut Vec<ChatMessage>,
    working_dir: &Path,
    model: &str,
    behavior: Option<&str>,
) {
    if messages.first().is_some_and(|m| m.role == "system") {
        return;
    }
    let tier = model_size::detect_tier(model);
    let prompt = match tier {
        PromptTier::Compact => prompt_chat_compact::build_with_behavior(working_dir, behavior),
        PromptTier::Detailed => prompt_chat_detailed::build_with_behavior(working_dir, behavior),
    };
    messages.insert(0, build_system_message(prompt));
}

fn append_response_language(messages: &mut [ChatMessage], lang: &str) {
    if lang.is_empty() {
        return;
    }
    let instruction = format!(
        "\n\nYou MUST respond in {lang}. All your answers, explanations and communications must be in {lang}."
    );
    if let Some(first) = messages.first_mut().filter(|m| m.role == "system") {
        first.content.push_str(&instruction);
    }
}

fn prepend_web_search_status(messages: &mut [ChatMessage]) {
    let section = super::web_search_status::format_web_search_status(
        crate::services::api_keys::has_key("brave"),
        crate::services::api_keys::has_key("exa"),
        crate::services::api_keys::has_key("firecrawl"),
        crate::services::searxng::is_ready(),
    );
    if let Some(first) = messages.first_mut().filter(|m| m.role == "system") {
        first.content.push_str(&section);
    }
}

fn prepend_skills_listing(messages: &mut [ChatMessage], skills: &[(String, String)]) {
    let listing = skills
        .iter()
        .map(|(name, desc)| format!("- {name}: {desc}"))
        .collect::<Vec<_>>()
        .join("\n");

    let section = format!(
        "\n\n## Available skills\n\
         The following skills are available. Use the `load_skill` tool to load one when relevant.\n\
         {listing}"
    );

    if let Some(first) = messages.first_mut().filter(|m| m.role == "system") {
        first.content.push_str(&section);
    }
}
