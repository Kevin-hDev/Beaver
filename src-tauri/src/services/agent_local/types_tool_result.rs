use serde::{Deserialize, Serialize};

use super::tool_result_contract::{ToolErrorCategory, ToolErrorInfo, ToolResultStatus};
use super::types_tool_result_details::ToolResultDetails;

const MAX_TOOL_WARNINGS: usize = 16;
const MAX_TOOL_WARNING_CHARS: usize = 1_000;

#[derive(Debug, Clone, Default)]
pub(super) struct ToolResultArtifacts {
    pub(super) ephemeral: Vec<super::tool_artifact::EphemeralArtifact>,
    pub(super) pending: Vec<super::tool_artifact::PendingArtifact>,
    pub(super) pending_resource: Option<super::tool_artifact::PendingExtensionResource>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub content: String,
    pub is_error: bool,
    #[serde(default)]
    pub status: ToolResultStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<ToolErrorInfo>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub warnings: Vec<String>,
    #[serde(default)]
    pub truncated: bool,
    #[serde(flatten)]
    pub(super) details: Box<ToolResultDetails>,
    #[serde(skip)]
    pub(super) artifacts: Box<ToolResultArtifacts>,
    #[serde(skip)]
    pub follow_up: Option<Box<ToolFollowUp>>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[must_use = "tool follow-ups must be recorded or explicitly handled"]
pub enum ToolFollowUp {
    #[default]
    None,
    UserMessage(String),
    SystemMessage(String),
    Stop,
}

impl ToolResult {
    pub fn ok(content: impl Into<String>) -> Self {
        Self::build(content, ToolResultStatus::Success, None)
    }

    pub fn error(
        content: impl Into<String>,
        code: &'static str,
        category: ToolErrorCategory,
        retryable: bool,
    ) -> Self {
        Self::build(
            content,
            ToolResultStatus::Error,
            Some(ToolErrorInfo::new(code, category, retryable)),
        )
    }

    pub fn partial<I, S>(content: impl Into<String>, warnings: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let mut result = Self::build(content, ToolResultStatus::Partial, None);
        for warning in warnings {
            result.push_warning(warning.into());
        }
        result
    }

    pub fn running(content: impl Into<String>) -> Self {
        Self::build(content, ToolResultStatus::Running, None)
    }

    pub fn cancelled(content: impl Into<String>) -> Self {
        Self::build(
            content,
            ToolResultStatus::Cancelled,
            Some(ToolErrorInfo::new(
                "tool_cancelled",
                ToolErrorCategory::Cancelled,
                false,
            )),
        )
    }

    pub fn stopped(content: impl Into<String>) -> Self {
        Self::build(content, ToolResultStatus::Stopped, None)
    }

    fn build(
        content: impl Into<String>,
        status: ToolResultStatus,
        error: Option<ToolErrorInfo>,
    ) -> Self {
        Self {
            content: content.into(),
            is_error: status.is_error(),
            status,
            error,
            warnings: Vec::new(),
            truncated: false,
            details: Box::default(),
            artifacts: Box::default(),
            follow_up: None,
        }
    }

    pub fn with_error_info(
        mut self,
        code: &'static str,
        category: ToolErrorCategory,
        retryable: bool,
    ) -> Self {
        self.status = ToolResultStatus::Error;
        self.is_error = true;
        self.error = Some(ToolErrorInfo::new(code, category, retryable));
        self
    }

    pub fn with_error_hint(mut self, hint: impl Into<String>) -> Self {
        if let Some(error) = self.error.take() {
            self.error = Some(error.with_hint(hint));
        }
        self
    }

    pub fn with_warning(mut self, warning: impl Into<String>) -> Self {
        self.push_warning(warning.into());
        if self.status == ToolResultStatus::Success {
            self.status = ToolResultStatus::Partial;
        }
        self
    }

    pub fn mark_truncated(&mut self, truncated: bool) {
        if !truncated {
            return;
        }
        self.truncated = true;
        if self.status == ToolResultStatus::Success {
            self.status = ToolResultStatus::Partial;
        }
    }

    fn push_warning(&mut self, warning: String) {
        if self.warnings.len() < MAX_TOOL_WARNINGS {
            self.warnings.push(bound_warning(warning));
        }
    }

    pub fn with_user_message(mut self, content: impl Into<String>) -> Self {
        self.follow_up = Some(Box::new(ToolFollowUp::UserMessage(content.into())));
        self
    }

    pub fn with_system_message(mut self, content: impl Into<String>) -> Self {
        self.follow_up = Some(Box::new(ToolFollowUp::SystemMessage(content.into())));
        self
    }

    pub fn stopping(mut self) -> Self {
        self.follow_up = Some(Box::new(ToolFollowUp::Stop));
        self
    }

    pub fn take_follow_up(&mut self) -> ToolFollowUp {
        self.follow_up
            .take()
            .map(|follow_up| *follow_up)
            .unwrap_or_default()
    }
}

fn bound_warning(warning: String) -> String {
    super::tool_result_contract::bound_safe_text(warning, MAX_TOOL_WARNING_CHARS)
}

#[cfg(test)]
#[path = "types_tool_result_tests.rs"]
mod tests;
