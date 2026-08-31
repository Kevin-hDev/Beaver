use crate::services::agent_local::types_session::AgentMessage;

#[derive(Debug, Clone)]
pub enum SelectedCheckpointMessage {
    Exact {
        source_index: usize,
        message: AgentMessage,
    },
    ToolResultExcerpt {
        source_index: usize,
        message: AgentMessage,
    },
}

impl SelectedCheckpointMessage {
    pub fn source_index(&self) -> usize {
        match self {
            Self::Exact { source_index, .. } | Self::ToolResultExcerpt { source_index, .. } => {
                *source_index
            }
        }
    }

    pub fn message(&self) -> &AgentMessage {
        match self {
            Self::Exact { message, .. } | Self::ToolResultExcerpt { message, .. } => message,
        }
    }
}

pub fn exact(source_index: usize, message: &AgentMessage) -> SelectedCheckpointMessage {
    SelectedCheckpointMessage::Exact {
        source_index,
        message: message.clone(),
    }
}

pub fn bounded_excerpt(input: &str, max_tokens: u32, marker: &str, reference: &str) -> String {
    let max_chars = usize::try_from(max_tokens)
        .unwrap_or(usize::MAX)
        .saturating_mul(4);
    let fixed = marker
        .chars()
        .count()
        .saturating_add(reference.chars().count());
    if max_chars <= fixed {
        return marker
            .chars()
            .chain(reference.chars())
            .take(max_chars)
            .collect();
    }
    let available = max_chars.saturating_sub(fixed);
    let beginning = available.div_ceil(2);
    let end = available.saturating_sub(beginning);
    let prefix = input.chars().take(beginning).collect::<String>();
    let suffix = input
        .chars()
        .rev()
        .take(end)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect::<String>();
    format!("{prefix}{marker}{suffix}{reference}")
}
