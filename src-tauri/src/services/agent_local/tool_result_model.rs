use serde::Serialize;

use super::tool_result_contract::{ToolErrorInfo, ToolResultStatus};
use super::types_tools::ToolResult;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ModelToolResult<'a> {
    kind: &'static str,
    tool: &'a str,
    status: ToolResultStatus,
    output: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<&'a ToolErrorInfo>,
    #[serde(skip_serializing_if = "slice_is_empty")]
    warnings: &'a [String],
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    truncated: bool,
}

pub fn render(tool_name: &str, result: &ToolResult) -> String {
    if result.status == ToolResultStatus::Success
        && !result.is_error
        && result.error.is_none()
        && result.warnings.is_empty()
        && !result.truncated
    {
        return result.content.clone();
    }

    serde_json::to_string(&ModelToolResult {
        kind: "tool_result",
        tool: tool_name,
        status: result.status,
        output: &result.content,
        error: result.error.as_ref(),
        warnings: &result.warnings,
        truncated: result.truncated,
    })
    .unwrap_or_else(|_| {
        format!(
            "[tool_result status={}] {}",
            result.status.as_str(),
            result.content
        )
    })
}

fn slice_is_empty<T>(slice: &&[T]) -> bool {
    slice.is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::agent_local::tool_result_contract::ToolErrorCategory;

    #[test]
    fn ordinary_success_keeps_its_exact_content() {
        assert_eq!(render("read_file", &ToolResult::ok("hello")), "hello");
    }

    #[test]
    fn error_exposes_machine_readable_status_to_the_model() {
        let result = ToolResult::error(
            "command output",
            "shell_exit_nonzero",
            ToolErrorCategory::Execution,
            false,
        );
        let rendered: serde_json::Value =
            serde_json::from_str(&render("bash", &result)).expect("model envelope");

        assert_eq!(rendered["status"], "error");
        assert_eq!(rendered["error"]["code"], "shell_exit_nonzero");
        assert_eq!(rendered["output"], "command output");
    }

    #[test]
    fn partial_result_exposes_warnings() {
        let result = ToolResult::partial("some files", ["one file was unreadable"]);
        let rendered: serde_json::Value =
            serde_json::from_str(&render("grep", &result)).expect("model envelope");

        assert_eq!(rendered["status"], "partial");
        assert_eq!(rendered["warnings"][0], "one file was unreadable");
    }

    #[test]
    fn running_process_is_not_serialized_as_a_completed_success() {
        let rendered: serde_json::Value =
            serde_json::from_str(&render("bash", &ToolResult::running("session=123")))
                .expect("model envelope");

        assert_eq!(rendered["status"], "running");
        assert_eq!(rendered["output"], "session=123");
    }
}
