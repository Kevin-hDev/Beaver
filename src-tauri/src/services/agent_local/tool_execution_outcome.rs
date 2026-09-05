use super::types_ollama::ChatMessage;
use super::tool_artifact::EphemeralArtifact;
use super::tool_execution_artifacts::{AttributedArtifact, ToolExecutionArtifacts};
use super::types_tools::ToolFollowUp;

pub(super) const MAX_FOLLOW_UP_BYTES: usize = 16 * 1024;

#[derive(Default)]
pub struct ToolExecutionOutcome {
    pub compressed: bool,
    follow_ups: Vec<ToolFollowUp>,
    artifacts: ToolExecutionArtifacts,
}

impl ToolExecutionOutcome {
    pub fn with_compressed(compressed: bool) -> Self {
        Self {
            compressed,
            follow_ups: Vec::new(),
            artifacts: ToolExecutionArtifacts::default(),
        }
    }

    pub fn record(&mut self, follow_up: ToolFollowUp) {
        if follow_up != ToolFollowUp::None {
            self.follow_ups.push(follow_up);
        }
    }

    pub fn merge(&mut self, other: Self) -> Result<(), ()> {
        // Keep an overflow atomic: callers stop the turn instead of publishing an
        // outcome whose visible message and retained artifacts disagree.
        self.artifacts.merge(other.artifacts)?;
        self.compressed |= other.compressed;
        self.follow_ups.extend(other.follow_ups);
        Ok(())
    }

    pub(crate) fn record_artifacts(
        &mut self,
        tool_call_index: usize,
        tool_call_id: Option<&str>,
        artifacts: Vec<EphemeralArtifact>,
    ) -> Result<(), ()> {
        self.artifacts.record(tool_call_index, tool_call_id, artifacts)
    }

    pub(crate) fn retain_artifacts(
        &mut self,
        tool_call_index: usize,
        tool_call_id: Option<&str>,
        artifacts: Vec<EphemeralArtifact>,
    ) -> Result<Vec<super::tool_artifact_record::ToolArtifactRecord>, ()> {
        let records = artifacts
            .iter()
            .map(|artifact| (&artifact.metadata).into())
            .collect();
        self.record_artifacts(tool_call_index, tool_call_id, artifacts)?;
        Ok(records)
    }

    pub(crate) fn artifacts(&self) -> &[AttributedArtifact] {
        self.artifacts.as_slice()
    }

    pub(crate) async fn take_artifact_previews(
        &mut self,
    ) -> super::tool_artifact_preview::ToolResultPreviewBatch {
        super::tool_artifact_preview::ToolResultPreviewBatch::replay_from_artifacts(
            self.artifacts.take(),
        )
        .await
    }

    pub fn apply_follow_ups(&mut self, messages: &mut [ChatMessage]) -> Result<bool, String> {
        let mut stop = false;
        for follow_up in std::mem::take(&mut self.follow_ups) {
            match follow_up {
                ToolFollowUp::None => {}
                ToolFollowUp::UserMessage(content) => {
                    append_to_tool(messages, "User follow-up", &content);
                }
                ToolFollowUp::SystemMessage(content) => {
                    append_to_tool(messages, "System follow-up", &content);
                }
                ToolFollowUp::Stop => stop = true,
            }
        }
        Ok(stop)
    }
}

fn append_to_tool(messages: &mut [ChatMessage], label: &str, content: &str) {
    let Some(tool) = messages
        .iter_mut()
        .rev()
        .find(|message| message.role == "tool")
    else {
        log::warn!("tool_follow_up_without_tool_message");
        return;
    };
    if content.is_empty() {
        return;
    }
    let content = bounded_prefix(content, MAX_FOLLOW_UP_BYTES);
    tool.content.push_str("\n\n");
    tool.content.push_str(label);
    tool.content.push_str(":\n");
    tool.content.push_str(content);
}

fn bounded_prefix(value: &str, max_bytes: usize) -> &str {
    if value.len() <= max_bytes {
        return value;
    }
    let end = value
        .char_indices()
        .map(|(index, _)| index)
        .take_while(|index| *index <= max_bytes)
        .last()
        .unwrap_or(0);
    &value[..end]
}

#[cfg(test)]
#[path = "tool_execution_outcome_tests.rs"]
mod tests;
