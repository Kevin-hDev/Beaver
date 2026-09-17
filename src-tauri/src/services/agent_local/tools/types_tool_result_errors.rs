use super::tool_result_contract::ToolErrorCategory;
use super::types_tool_result::ToolResult;

impl ToolResult {
    pub fn validation(code: &'static str, content: impl Into<String>) -> Self {
        Self::error(content, code, ToolErrorCategory::Validation, false)
    }

    pub fn permission(code: &'static str, content: impl Into<String>) -> Self {
        Self::error(content, code, ToolErrorCategory::Permission, false)
    }

    pub fn not_found(code: &'static str, content: impl Into<String>) -> Self {
        Self::error(content, code, ToolErrorCategory::NotFound, false)
    }

    pub fn conflict(code: &'static str, content: impl Into<String>) -> Self {
        Self::error(content, code, ToolErrorCategory::Conflict, false)
    }

    pub fn timeout(
        code: &'static str,
        content: impl Into<String>,
        retryable: bool,
    ) -> Self {
        Self::error(content, code, ToolErrorCategory::Timeout, retryable)
    }

    pub fn unavailable(
        code: &'static str,
        content: impl Into<String>,
        retryable: bool,
    ) -> Self {
        Self::error(content, code, ToolErrorCategory::Unavailable, retryable)
    }

    pub fn external(
        code: &'static str,
        content: impl Into<String>,
        retryable: bool,
    ) -> Self {
        Self::error(content, code, ToolErrorCategory::External, retryable)
    }

    pub fn execution(
        code: &'static str,
        content: impl Into<String>,
        retryable: bool,
    ) -> Self {
        Self::error(content, code, ToolErrorCategory::Execution, retryable)
    }

    pub fn internal(
        code: &'static str,
        content: impl Into<String>,
        retryable: bool,
    ) -> Self {
        Self::error(content, code, ToolErrorCategory::Internal, retryable)
    }
}
