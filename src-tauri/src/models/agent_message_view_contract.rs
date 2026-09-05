use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::ReasoningReplayStatus;
use crate::services::agent_local::tool_artifact_record::{
    ToolArtifactPurpose, ToolArtifactRecord, ToolArtifactSource, ToolArtifactStatus,
};
use crate::services::agent_local::types_message::AgentMessageKind;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
#[serde(deny_unknown_fields)]
pub struct FileAttachmentView {
    pub name: String,
    pub path: String,
    pub mime_type: String,
    #[cfg_attr(test, ts(type = "number"))]
    pub size: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(test, ts(optional))]
    pub thumbnail: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(test, ts(optional))]
    pub access_grant: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
#[serde(deny_unknown_fields)]
pub struct ToolCallFunctionView {
    pub name: String,
    #[cfg_attr(test, ts(type = "Record<string, unknown>"))]
    pub arguments: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
#[serde(deny_unknown_fields)]
pub struct ToolCallRequestView {
    pub id: String,
    pub function: ToolCallFunctionView,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
#[serde(deny_unknown_fields)]
pub struct ToolFileChangeView {
    pub path: String,
    #[cfg_attr(test, ts(type = "\"added\" | \"modified\" | \"deleted\""))]
    pub status: String,
    pub additions: usize,
    pub deletions: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(test, ts(optional, type = "GitDiffPreview"))]
    pub diff: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
#[serde(tag = "kind", rename_all = "snake_case")]
#[cfg_attr(test, ts(tag = "kind", rename_all = "snake_case"))]
pub enum ToolArtifactSourceView {
    WorkspaceFile { path: String },
    ExtensionResource { resource_id: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
#[serde(deny_unknown_fields)]
pub struct ToolArtifactRecordView {
    pub name: String,
    pub mime_type: String,
    #[cfg_attr(test, ts(type = "number"))]
    pub bytes: u64,
    pub sha256: String,
    pub purpose: ToolArtifactPurpose,
    pub source: ToolArtifactSourceView,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(test, ts(optional))]
    pub verification: Option<ToolArtifactStatus>,
}

impl From<&ToolArtifactRecord> for ToolArtifactRecordView {
    fn from(record: &ToolArtifactRecord) -> Self {
        Self {
            name: record.name.clone(),
            mime_type: record.mime_type.clone(),
            bytes: record.bytes,
            sha256: record.sha256.clone(),
            purpose: record.purpose.clone(),
            source: match &record.source {
                ToolArtifactSource::WorkspaceFile { path, .. } => {
                    ToolArtifactSourceView::WorkspaceFile { path: path.clone() }
                }
                ToolArtifactSource::ExtensionResource { resource_id, .. } => {
                    ToolArtifactSourceView::ExtensionResource {
                        resource_id: resource_id.clone(),
                    }
                }
            },
            verification: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
#[serde(deny_unknown_fields)]
pub struct ToolActivityRecordView {
    pub name: String,
    pub summary: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(test, ts(optional, type = "\"memory\""))]
    pub domain: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(test, ts(optional))]
    pub resolved_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(test, ts(optional, type = "Record<string, unknown>"))]
    pub args: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(test, ts(optional))]
    pub result: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(test, ts(optional))]
    pub is_error: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(test, ts(optional, type = "PersistedToolResultMeta"))]
    pub result_meta: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(test, ts(optional))]
    pub content: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(test, ts(optional))]
    pub old_text: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(test, ts(optional))]
    pub new_text: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(test, ts(optional))]
    pub start_line: Option<u32>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[cfg_attr(test, ts(optional, as = "Option<_>"))]
    pub affected_paths: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[cfg_attr(test, ts(optional, as = "Option<_>"))]
    pub file_changes: Vec<ToolFileChangeView>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[cfg_attr(test, ts(optional, as = "Option<_>"))]
    pub artifacts: Vec<ToolArtifactRecordView>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
#[serde(deny_unknown_fields)]
pub struct SavedSegmentView {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(test, ts(optional))]
    pub thinking: Option<String>,
    pub tools: Vec<ToolActivityRecordView>,
    pub content: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(test, ts(optional, type = "\"work\" | \"final\""))]
    pub phase: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
#[serde(deny_unknown_fields)]
pub struct AgentMessageView {
    pub id: String,
    pub turn_id: String,
    #[cfg_attr(test, ts(type = "\"user\" | \"assistant\" | \"tool\""))]
    pub role: String,
    pub content: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(test, ts(optional))]
    pub message_kind: Option<AgentMessageKind>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(test, ts(optional))]
    pub thinking: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(test, ts(optional))]
    pub tool_calls: Option<Vec<ToolCallRequestView>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(test, ts(optional))]
    pub tool_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(test, ts(optional))]
    pub tool_call_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(test, ts(optional))]
    pub tool_activities: Option<Vec<ToolActivityRecordView>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(test, ts(optional))]
    pub segments: Option<Vec<SavedSegmentView>>,
    pub files: Vec<FileAttachmentView>,
    #[cfg_attr(test, ts(type = "string"))]
    pub timestamp: DateTime<Utc>,
    #[serde(default)]
    pub tokens: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(test, ts(optional, type = "number"))]
    pub work_duration_ms: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(test, ts(optional))]
    pub skill_names: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(test, ts(optional))]
    pub stream_run_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(test, ts(optional, type = "\"checkpoint\" | \"input\" | \"final\""))]
    pub stream_part: Option<String>,
    pub reasoning_replay_status: ReasoningReplayStatus,
}
