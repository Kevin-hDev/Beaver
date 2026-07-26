use serde::{Deserialize, Serialize};

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
pub struct ToolResult {
    pub content: String,
    pub is_error: bool,
    #[serde(default)]
    pub truncated: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub display_summary: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub affected_paths: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub file_changes: Vec<ToolFileChange>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start_line: Option<usize>,
}

impl ToolResult {
    pub fn ok(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            is_error: false,
            truncated: false,
            display_summary: None,
            affected_paths: Vec::new(),
            file_changes: Vec::new(),
            start_line: None,
        }
    }

    pub fn err(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            is_error: true,
            truncated: false,
            display_summary: None,
            affected_paths: Vec::new(),
            file_changes: Vec::new(),
            start_line: None,
        }
    }

    pub fn with_affected_paths(mut self, paths: Vec<String>) -> Self {
        self.affected_paths = paths;
        self
    }

    pub fn with_display_summary(mut self, summary: impl Into<String>) -> Self {
        self.display_summary = Some(summary.into());
        self
    }

    pub fn with_file_changes(mut self, changes: Vec<ToolFileChange>) -> Self {
        self.file_changes = changes;
        self
    }

    pub fn with_start_line(mut self, start_line: usize) -> Self {
        self.start_line = Some(start_line);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShellOutput {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
    pub timed_out: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub new_cwd: Option<String>,
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
