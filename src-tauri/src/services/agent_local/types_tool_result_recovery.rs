use super::{ToolFollowUp, ToolResult, ToolResultArtifacts, ToolResultDetails};
use crate::services::agent_local::stream_recovery_record::{
    RecoverableToolFollowUp, RecoverableToolResult,
};

impl ToolResult {
    pub(crate) fn from_recovery(snapshot: &RecoverableToolResult) -> Self {
        let follow_up = match &snapshot.follow_up {
            RecoverableToolFollowUp::None => None,
            RecoverableToolFollowUp::UserMessage(value) => {
                Some(Box::new(ToolFollowUp::UserMessage(value.clone())))
            }
            RecoverableToolFollowUp::SystemMessage(value) => {
                Some(Box::new(ToolFollowUp::SystemMessage(value.clone())))
            }
            RecoverableToolFollowUp::Stop => Some(Box::new(ToolFollowUp::Stop)),
        };
        Self {
            content: snapshot.content.clone(),
            is_error: snapshot.is_error,
            status: snapshot.status,
            error: snapshot.error.clone(),
            warnings: snapshot.warnings.clone(),
            truncated: snapshot.truncated,
            details: Box::new(ToolResultDetails {
                display_summary: snapshot
                    .display_summary
                    .clone()
                    .map(String::into_boxed_str),
                affected_paths: snapshot.affected_paths.clone(),
                file_changes: snapshot.file_changes.clone(),
                start_line: snapshot.start_line,
            }),
            artifacts: Box::<ToolResultArtifacts>::default(),
            follow_up,
        }
    }

    pub(crate) fn persistence_snapshot(
        &self,
        name: &str,
        tool_call_index: usize,
        tool_call_id: Option<&str>,
        resolved_path: Option<String>,
        domain: Option<String>,
        artifacts: Vec<crate::services::agent_local::tool_artifact_record::ToolArtifactRecord>,
    ) -> RecoverableToolResult {
        let follow_up = match self.follow_up.as_deref() {
            None | Some(ToolFollowUp::None) => RecoverableToolFollowUp::None,
            Some(ToolFollowUp::UserMessage(value)) => {
                RecoverableToolFollowUp::UserMessage(value.clone())
            }
            Some(ToolFollowUp::SystemMessage(value)) => {
                RecoverableToolFollowUp::SystemMessage(value.clone())
            }
            Some(ToolFollowUp::Stop) => RecoverableToolFollowUp::Stop,
        };
        RecoverableToolResult {
            message_id: uuid::Uuid::new_v4().to_string(),
            name: name.to_string(),
            content: self.content.clone(),
            is_error: self.is_error,
            status: self.status,
            error: self.error.clone(),
            warnings: self.warnings.clone(),
            truncated: self.truncated,
            display_summary: self.display_summary().map(str::to_owned),
            tool_call_index,
            tool_call_id: tool_call_id.map(str::to_owned),
            resolved_path,
            domain,
            affected_paths: self.affected_paths().to_vec(),
            file_changes: self.file_changes().to_vec(),
            start_line: self.start_line(),
            artifacts,
            follow_up,
        }
    }
}
