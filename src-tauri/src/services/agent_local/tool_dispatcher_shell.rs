use serde_json::Value;
use std::path::Path;
use tokio_util::sync::CancellationToken;

use super::subagent_tool_profile::SubagentToolProfile;
use super::tool_bash_progress::ShellProgress;
use super::types_tools::{ShellOutput, ToolResult};

pub async fn dispatch(
    tool_name: &str,
    args: &Value,
    working_dir: &Path,
    session_id: &str,
    cancel: CancellationToken,
    profile: Option<SubagentToolProfile>,
    progress: Option<ShellProgress>,
) -> ToolResult {
    let execution = match tool_name {
        "bash" => execute_command(args, working_dir, session_id, cancel, profile, progress).await,
        "bash_write" => control_process(args, session_id, cancel, progress).await,
        _ => return ToolResult::err("Outil shell inconnu."),
    };
    execution.map(to_tool_result).unwrap_or_else(ToolResult::err)
}

async fn execute_command(
    args: &Value,
    working_dir: &Path,
    session_id: &str,
    cancel: CancellationToken,
    profile: Option<SubagentToolProfile>,
    progress: Option<ShellProgress>,
) -> Result<ShellOutput, String> {
    let command = args["command"].as_str().unwrap_or("");
    let timeout = args["timeout"].as_u64();
    let execution_dir = super::tool_bash::resolve_workdir(args["workdir"].as_str(), working_dir)?;
    if profile == Some(SubagentToolProfile::Explorer) {
        return super::subagent_explorer_bash::execute(command, &execution_dir, timeout, cancel)
            .await;
    }
    super::tool_bash::execute_shell_managed(
        command,
        &execution_dir,
        super::tool_bash::ShellExecutionContext {
            owner_session_id: session_id,
            hard_timeout_secs: timeout,
            yield_time_ms: yield_time_ms(args),
            cancel,
            progress,
        },
    )
    .await
}

async fn control_process(
    args: &Value,
    session_id: &str,
    cancel: CancellationToken,
    progress: Option<ShellProgress>,
) -> Result<ShellOutput, String> {
    super::tool_bash::control_shell_session(
        args["session_id"].as_str().unwrap_or(""),
        args["chars"].as_str(),
        args["stop"].as_bool().unwrap_or(false),
        session_id,
        yield_time_ms(args),
        cancel,
        progress,
    )
    .await
}

fn yield_time_ms(args: &Value) -> Option<u64> {
    args["yield_time_ms"]
        .as_u64()
        .or_else(|| args["yield-time-ms"].as_u64())
}

fn to_tool_result(output: ShellOutput) -> ToolResult {
    let content = format!("{}\n{}", output.stdout, output.stderr)
        .trim()
        .to_string();
    let result = if output.exit_code == 0 {
        ToolResult::ok(content)
    } else {
        ToolResult::err(content)
    };
    result
        .with_affected_paths(output.affected_paths)
        .with_file_changes(output.file_changes)
}
