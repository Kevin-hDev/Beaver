use std::path::Path;

use crate::services::agent_local::types_ollama::ChatMessage;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct CheckpointFile {
    pub tool: String,
    pub path: String,
    pub current_content: String,
}

pub async fn collect_with_budget(
    messages: &[ChatMessage],
    working_dir: &Path,
    budget: super::checkpoint_evidence::EvidenceItemLimit,
) -> Vec<CheckpointFile> {
    collect_kind(messages, working_dir, budget, FileKind::Any).await
}

#[derive(Clone, Copy)]
enum FileKind {
    Any,
}

async fn collect_kind(
    messages: &[ChatMessage],
    working_dir: &Path,
    budget: super::checkpoint_evidence::EvidenceItemLimit,
    kind: FileKind,
) -> Vec<CheckpointFile> {
    if budget.max_items == 0 || budget.tokens_per_item == 0 {
        return Vec::new();
    }
    let max_items = usize::from(budget.max_items);
    let mut remaining = if budget.total_tokens == 0 {
        u32::MAX
    } else {
        budget.total_tokens
    };
    let mut output = Vec::new();
    for event in super::context_capsules_disk_collect::recent_disk_file_events_bounded(
        messages,
        working_dir,
        usize::from(super::profile_limits::MAX_FILES),
        budget.tokens_per_item,
    )
    .await
    {
        let _ = kind;
        if output.len() >= max_items {
            break;
        }
        let tokens = crate::services::token_counting::estimate_text_tokens(&event.result)
            .min(u32::MAX as usize) as u32;
        if tokens > remaining {
            continue;
        }
        remaining = remaining.saturating_sub(tokens);
        output.push(CheckpointFile {
            tool: event.tool,
            path: event.path,
            current_content: event.result,
        });
    }
    output
}
