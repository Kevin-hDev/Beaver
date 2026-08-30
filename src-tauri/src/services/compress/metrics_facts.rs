use super::checkpoint_candidate::CompressionCandidate;
use super::metrics::CompressionSuccessFacts;
use super::snapshot::CompressionSnapshot;
use super::summary_contract::ValidatedSummary;
use crate::services::agent_local::types_message::{AgentMessage, AgentMessageKind};

pub fn collect(
    snapshot: &CompressionSnapshot,
    summary: Option<&ValidatedSummary>,
    candidate: &CompressionCandidate,
    compression_count: u32,
) -> CompressionSuccessFacts {
    let source_tools = count_role(&snapshot.source_messages, "tool");
    let retained_tools = count_role(&candidate.persisted_messages, "tool");
    let source_images = snapshot
        .source_messages
        .iter()
        .flat_map(|message| &message.files)
        .filter(|file| file.thumbnail.is_some())
        .count();
    let retained_images = snapshot.checkpoint_images.len();
    CompressionSuccessFacts {
        after_tokens: candidate.after_tokens,
        summary_tokens: summary.map_or(0, |value| text_tokens(&value.content)),
        retained_user_tokens: candidate
            .persisted_messages
            .iter()
            .filter(|message| message.role == "user" && message.message_kind.is_none())
            .fold(0u32, |total, message| {
                total.saturating_add(super::token_estimate::estimate_checkpoint_message_tokens(
                    message,
                ))
            }),
        retained_tool_results: count_u16(retained_tools),
        dropped_tool_results: count_u16(source_tools.saturating_sub(retained_tools)),
        retained_images: count_u16(retained_images),
        dropped_images: count_u16(source_images.saturating_sub(retained_images)),
        retained_subagent_reports: checkpoint_report_count(&candidate.persisted_messages),
        compression_count,
    }
}

fn checkpoint_report_count(messages: &[AgentMessage]) -> u16 {
    let Some(content) = messages.iter().find_map(|message| {
        (message.message_kind == Some(AgentMessageKind::CompressionCheckpoint))
            .then_some(message.content.as_str())
    }) else {
        return 0;
    };
    let Ok(body) = serde_json::from_str::<serde_json::Value>(content) else {
        return 0;
    };
    body.pointer("/sections/subagents")
        .and_then(serde_json::Value::as_str)
        .and_then(|value| serde_json::from_str::<serde_json::Value>(value).ok())
        .and_then(|value| value.get("pending_reports")?.as_array().map(Vec::len))
        .map_or(0, count_u16)
}

fn count_role(messages: &[AgentMessage], role: &str) -> usize {
    messages
        .iter()
        .filter(|message| message.role == role)
        .count()
}

fn count_u16(value: usize) -> u16 {
    value.min(usize::from(u16::MAX)) as u16
}

fn text_tokens(value: &str) -> u32 {
    crate::services::token_counting::estimate_text_tokens(value).min(u32::MAX as usize) as u32
}
