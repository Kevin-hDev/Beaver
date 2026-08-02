use serde_json::Value;
use std::path::Path;
use tokio_util::sync::CancellationToken;

use super::subagent_tool_profile::SubagentToolProfile;
use super::tool_bash_progress::ShellProgress;
use super::tool_result_contract::ToolErrorCategory;
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
        "bash" => execute_command(args, working_dir, session_id, cancel, profile, progress)
            .await
            .map(to_tool_result),
        "bash_control" => control_process(args, session_id, cancel, progress)
            .await
            .map(|(output, command)| {
                to_tool_result(output).with_display_summary(command.as_str())
            }),
        _ => {
            return ToolResult::error(
                "Outil shell inconnu.",
                "unknown_shell_tool",
                ToolErrorCategory::Validation,
                false,
            )
        }
    };
    execution.unwrap_or_else(super::tool_dispatcher_shell_error::from_message)
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
) -> Result<(ShellOutput, super::tool_bash_registry::RegisteredCommand), String> {
    let process_id = args["session_id"].as_str().unwrap_or("");
    super::tool_bash::control_shell_session(
        process_id,
        args["chars"].as_str(),
        args["eof"].as_bool().unwrap_or(false),
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
    let mut content = render_streams(&output.stdout, &output.stderr);
    if !output.running && !output.stopped && output.exit_code > 0 {
        append_note(&mut content, &format!("[Code de sortie: {}]", output.exit_code));
    }
    let mut result = if output.running {
        ToolResult::running(content)
    } else if output.stopped {
        ToolResult::stopped(content)
    } else if output.cancelled {
        ToolResult::cancelled(content)
    } else if output.blocked {
        ToolResult::error(
            content,
            "shell_command_blocked",
            ToolErrorCategory::Permission,
            false,
        )
    } else if output.timed_out {
        ToolResult::error(
            content,
            "shell_timeout",
            ToolErrorCategory::Timeout,
            false,
        )
        .with_error_hint(
            "Vérifier l'état du projet avant de relancer : la commande a pu effectuer une partie de son travail.",
        )
    } else if output.exit_code == 0 {
        ToolResult::ok(content)
    } else if output.exit_code > 0 {
        ToolResult::error(
            content,
            "shell_exit_nonzero",
            ToolErrorCategory::Execution,
            false,
        )
    } else {
        ToolResult::error(
            content,
            "shell_execution_failed",
            ToolErrorCategory::Execution,
            false,
        )
        .with_error_hint(
            "Vérifier l'état du projet avant de relancer : la commande a pu modifier des fichiers.",
        )
    };
    if output.tracking_incomplete {
        result = result.with_warning("Le suivi des fichiers modifiés est incomplet.");
    }
    if output.output_incomplete {
        let warning = if output.exit_code == 0 {
            "La commande a réussi, mais un processus détaché conserve les sorties ouvertes."
        } else {
            "La commande est terminée, mais un processus détaché conserve les sorties ouvertes."
        };
        result = result.with_warning(warning);
    }
    result.mark_truncated(output.output_truncated);
    result
        .with_affected_paths(output.affected_paths)
        .with_file_changes(output.file_changes)
}

fn append_note(content: &mut String, note: &str) {
    if !content.is_empty() {
        content.push_str("\n\n");
    }
    content.push_str(note);
}

fn render_streams(stdout: &str, stderr: &str) -> String {
    match (stdout.trim(), stderr.trim()) {
        ("", "") => String::new(),
        (stdout, "") => stdout.to_string(),
        ("", stderr) => stderr.to_string(),
        (stdout, stderr) => format!("{stdout}\n\n[stderr]\n{stderr}"),
    }
}

#[cfg(test)]
#[path = "tool_dispatcher_shell_tests.rs"]
mod tests;
