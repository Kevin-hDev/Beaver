#![allow(
    dead_code,
    reason = "the compression orchestrator consumes checkpoint files in Task 10"
)]

use std::path::Path;

use crate::services::agent_local::types_ollama::ChatMessage;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckpointFile {
    pub tool: String,
    pub path: String,
    pub current_content: String,
}

pub async fn collect(
    messages: &[ChatMessage],
    working_dir: &Path,
    context_window: u64,
) -> Vec<CheckpointFile> {
    let max_files = if context_window < 64_000 {
        super::context_capsules_disk::MAX_UNDER_64K_FILES
    } else {
        super::context_capsules_disk::MAX_FILES
    };
    super::context_capsules_disk_collect::recent_disk_file_events(messages, working_dir, max_files)
        .await
        .into_iter()
        .map(|event| CheckpointFile {
            tool: event.tool,
            path: event.path,
            current_content: event.result,
        })
        .collect()
}
