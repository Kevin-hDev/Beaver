use serde::{Deserialize, Serialize};

use super::types_tool_result::ToolResult;
use super::types_tools::ToolFileChange;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub(super) struct ToolResultDetails {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    display_summary: Option<Box<str>>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    affected_paths: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    file_changes: Vec<ToolFileChange>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    start_line: Option<usize>,
}

impl ToolResult {
    pub fn with_affected_paths(mut self, paths: Vec<String>) -> Self {
        self.details.affected_paths = paths;
        self
    }

    pub fn with_display_summary(mut self, summary: impl Into<String>) -> Self {
        self.details.display_summary = Some(summary.into().into_boxed_str());
        self
    }

    pub fn with_file_changes(mut self, changes: Vec<ToolFileChange>) -> Self {
        self.details.file_changes = changes;
        self
    }

    pub fn with_start_line(mut self, start_line: usize) -> Self {
        self.details.start_line = Some(start_line);
        self
    }

    pub fn display_summary(&self) -> Option<&str> {
        self.details.display_summary.as_deref()
    }

    pub fn affected_paths(&self) -> &[String] {
        &self.details.affected_paths
    }

    pub fn affected_paths_mut(&mut self) -> &mut Vec<String> {
        &mut self.details.affected_paths
    }

    pub fn file_changes(&self) -> &[ToolFileChange] {
        &self.details.file_changes
    }

    pub fn file_changes_mut(&mut self) -> &mut Vec<ToolFileChange> {
        &mut self.details.file_changes
    }

    pub fn start_line(&self) -> Option<usize> {
        self.details.start_line
    }
}
