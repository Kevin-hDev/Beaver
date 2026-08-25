use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::{FileAttachmentView, SavedSegmentView, ToolActivityRecordView, ToolCallRequestView};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
#[serde(deny_unknown_fields)]
pub struct VisibleMessageInput {
    pub id: String,
    #[cfg_attr(test, ts(type = "\"user\" | \"assistant\" | \"tool\""))]
    pub role: String,
    pub content: String,
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
}
