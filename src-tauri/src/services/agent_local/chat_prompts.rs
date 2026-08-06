use crate::services::agent_local::types_ollama::ChatMessage;
use super::chat_prompt_sections::{response_language_instruction, skills_listing_section};
use super::system_prompt_types::{PromptSource, SystemPromptView};
#[cfg(test)]
use super::system_prompt_types::{PromptMode, PromptSelection, PromptTier};
#[cfg(test)]
use crate::services::agent_local::model_size;
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

pub(crate) fn compose_instructions_with_runtime(
    mode: &str,
    working_dir: &Path,
    is_git: bool,
    git_root: Option<&Path>,
    instructions: &SystemPromptView,
    enabled_tool_names: &[String],
) -> String {
    let mut prompt = instructions.content.clone();
    if instructions.source == PromptSource::Beaver {
        prompt = super::tool_prompt_filter::filter_system_prompt(&prompt, enabled_tool_names);
    }
    let runtime = if mode == "chat" {
        super::system_prompt_runtime_context::chatbot_environment(working_dir)
    } else {
        super::system_prompt_runtime_context::agentic_environment(working_dir, is_git, git_root)
    };
    append_section(&mut prompt, &runtime);
    prompt
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
    let prompt_mode = if mode == "chat" {
        PromptMode::Chatbot
    } else {
        PromptMode::Agentic
    };
    let prompt_tier = match model_size::detect_tier(model) {
        model_size::PromptTier::Compact => PromptTier::Compact,
        model_size::PromptTier::Detailed => PromptTier::Detailed,
    };
    let instructions = SystemPromptView {
        content: super::system_prompt_defaults::beaver_prompt(prompt_mode, prompt_tier),
        source: PromptSource::Beaver,
        selection: PromptSelection::Default,
        disabled: false,
    };
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
        &instructions,
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
    _model: &str,
    mode: &str,
    response_language: &str,
    enabled_tool_names: &[String],
    instructions: &SystemPromptView,
) {
    if mode == "chat" {
        prepend_chat_system_prompt(messages, working_dir, instructions, enabled_tool_names);
    } else {
        let prompt = compose_instructions_with_runtime(
            mode,
            working_dir,
            is_git,
            git_root,
            instructions,
            enabled_tool_names,
        );
        if !messages.first().is_some_and(|message| message.role == "system") {
            messages.insert(0, build_system_message(prompt));
        }
        super::extension_discovery_prompt::append(messages, enabled_tool_names);
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
    instructions: &SystemPromptView,
    enabled_tool_names: &[String],
) {
    if messages.first().is_some_and(|m| m.role == "system") {
        return;
    }
    let prompt = compose_instructions_with_runtime(
        "chat",
        working_dir,
        false,
        None,
        instructions,
        enabled_tool_names,
    );
    messages.insert(0, build_system_message(prompt));
}

fn append_section(prompt: &mut String, section: &str) {
    if !prompt.is_empty() {
        prompt.push_str("\n\n");
    }
    prompt.push_str(section);
}

fn append_response_language(messages: &mut [ChatMessage], lang: &str) {
    let Some(instruction) = response_language_instruction(lang) else {
        return;
    };
    if let Some(first) = messages.first_mut().filter(|m| m.role == "system") {
        first.content.push_str(&instruction);
    }
}

fn prepend_skills_listing(messages: &mut [ChatMessage], skills: &[(String, String)]) {
    let Some(section) = skills_listing_section(skills) else {
        return;
    };
    if let Some(first) = messages.first_mut().filter(|m| m.role == "system") {
        first.content.push_str(&section);
    }
}
