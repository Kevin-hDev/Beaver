use crate::services::agent_local::types_ollama::ChatMessage;
use crate::services::agent_local::types_session::AgentMessage;

const RECENT_PER_ROLE: usize = 2;

pub fn select_chat_tail(messages: &[ChatMessage]) -> Vec<ChatMessage> {
    let selected = selected_role_indices(messages, |message| {
        super::state_recent::include_chat_message(message).then_some(message.role.as_str())
    });
    expand_chat_tool_results(messages, &selected)
}

pub fn select_agent_tail(messages: &[AgentMessage]) -> Vec<AgentMessage> {
    let selected = selected_role_indices(messages, |message| {
        super::state_recent::include_agent_message(message).then_some(message.role.as_str())
    });
    expand_agent_tool_results(messages, &selected)
}

fn selected_role_indices<T>(messages: &[T], role: impl Fn(&T) -> Option<&str>) -> Vec<bool> {
    let mut selected = vec![false; messages.len()];
    let mut users = 0usize;
    let mut assistants = 0usize;
    for (idx, message) in messages.iter().enumerate().rev() {
        match role(message) {
            Some("user") if users < RECENT_PER_ROLE => {
                selected[idx] = true;
                users += 1;
            }
            Some("assistant") if assistants < RECENT_PER_ROLE => {
                selected[idx] = true;
                assistants += 1;
            }
            _ => {}
        }
        if users >= RECENT_PER_ROLE && assistants >= RECENT_PER_ROLE {
            break;
        }
    }
    selected
}

fn expand_chat_tool_results(messages: &[ChatMessage], selected: &[bool]) -> Vec<ChatMessage> {
    let mut expanded = selected.to_vec();
    for (idx, message) in messages.iter().enumerate() {
        let call_count = message.tool_calls.as_ref().map_or(0, Vec::len);
        if !selected[idx] || message.role != "assistant" || call_count == 0 {
            continue;
        }
        mark_following_tools(messages, &mut expanded, idx + 1, call_count);
    }
    collect_selected(messages, &expanded)
}

fn expand_agent_tool_results(messages: &[AgentMessage], selected: &[bool]) -> Vec<AgentMessage> {
    let mut expanded = selected.to_vec();
    for (idx, message) in messages.iter().enumerate() {
        let call_count = message.tool_calls.as_ref().map_or(0, Vec::len);
        if !selected[idx] || message.role != "assistant" || call_count == 0 {
            continue;
        }
        mark_following_tools(messages, &mut expanded, idx + 1, call_count);
    }
    collect_selected(messages, &expanded)
}

fn mark_following_tools<T: HasRole>(
    messages: &[T],
    selected: &mut [bool],
    start: usize,
    mut remaining: usize,
) {
    for idx in start..messages.len() {
        if remaining == 0 || messages[idx].role() != "tool" {
            break;
        }
        selected[idx] = true;
        remaining -= 1;
    }
}

fn collect_selected<T: Clone>(messages: &[T], selected: &[bool]) -> Vec<T> {
    messages
        .iter()
        .zip(selected)
        .filter_map(|(message, keep)| keep.then_some(message.clone()))
        .collect()
}

trait HasRole {
    fn role(&self) -> &str;
}

impl HasRole for ChatMessage {
    fn role(&self) -> &str {
        &self.role
    }
}

impl HasRole for AgentMessage {
    fn role(&self) -> &str {
        &self.role
    }
}
