use serde::{Deserialize, Serialize};

pub use super::types_tool_result::{ToolFollowUp, ToolResult};

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ToolFileChangeStatus {
    Added,
    Modified,
    Deleted,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ToolFileChange {
    pub path: String,
    pub status: ToolFileChangeStatus,
    pub additions: usize,
    pub deletions: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub diff: Option<crate::services::git::diff_preview::GitDiffPreview>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShellOutput {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
    #[serde(default)]
    pub running: bool,
    #[serde(default)]
    pub stopped: bool,
    #[serde(default)]
    pub cancelled: bool,
    #[serde(default)]
    pub blocked: bool,
    pub timed_out: bool,
    #[serde(default)]
    pub tracking_incomplete: bool,
    #[serde(default)]
    pub output_truncated: bool,
    #[serde(default)]
    pub output_incomplete: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sandbox_warning: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub affected_paths: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub file_changes: Vec<ToolFileChange>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub title: String,
    pub url: String,
    pub snippet: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillInfo {
    pub id: String,
    pub name: String,
    pub command: String,
    pub description: String,
    pub path: String,
    pub source: String,
    pub source_name: String,
}
