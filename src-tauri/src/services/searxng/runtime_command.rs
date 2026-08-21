use std::process::Stdio;
use std::time::Duration;

use tokio::process::Command;
use tokio::time::Instant;

use crate::services::work_registry::ServiceWorkCancellation;

use super::runtime_command_drain::{DrainTasks, DrainedOutput, OutputTail};

pub(super) const RUNTIME_INSTALL_TIMEOUT: Duration = Duration::from_secs(300);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum RuntimeStage {
    CreateVenv,
    InstallBuildTools,
    InstallRequirements,
    ValidateImports,
}

impl RuntimeStage {
    pub(super) fn as_str(self) -> &'static str {
        match self {
            Self::CreateVenv => "create-venv",
            Self::InstallBuildTools => "install-build-tools",
            Self::InstallRequirements => "install-requirements",
            Self::ValidateImports => "validate-imports",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum RuntimeCommandError {
    Spawn(RuntimeStage),
    Drain(RuntimeStage),
    NonZero(RuntimeStage, Option<i32>),
    Timeout(RuntimeStage),
    Cancelled(RuntimeStage),
    Diagnostics(RuntimeStage),
}

impl RuntimeCommandError {
    pub(super) fn stage(self) -> RuntimeStage {
        match self {
            Self::Spawn(stage)
            | Self::Drain(stage)
            | Self::NonZero(stage, _)
            | Self::Timeout(stage)
            | Self::Cancelled(stage)
            | Self::Diagnostics(stage) => stage,
        }
    }

    pub(super) fn category(self) -> &'static str {
        match self {
            Self::Spawn(_) => "spawn",
            Self::Drain(_) => "drain",
            Self::NonZero(_, _) => "non-zero",
            Self::Timeout(_) => "timeout",
            Self::Cancelled(_) => "cancelled",
            Self::Diagnostics(_) => "diagnostics",
        }
    }

    pub(super) fn exit_code(self) -> Option<i32> {
        match self {
            Self::NonZero(_, code) => code,
            _ => None,
        }
    }
}

pub(super) async fn run_runtime_command(
    command: &mut Command,
    stage: RuntimeStage,
    deadline: Instant,
    cancel: &ServiceWorkCancellation,
) -> Result<(), RuntimeCommandError> {
    if cancel.is_cancelled() {
        return record(
            RuntimeCommandError::Cancelled(stage),
            &OutputTail::new(),
            &OutputTail::new(),
        );
    }
    if Instant::now() >= deadline {
        return record(
            RuntimeCommandError::Timeout(stage),
            &OutputTail::new(),
            &OutputTail::new(),
        );
    }
    command
        .env("PIP_DISABLE_PIP_VERSION_CHECK", "1")
        .env("PIP_NO_INPUT", "1")
        .env("PYTHONUNBUFFERED", "1")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);
    let mut child = match crate::services::owned_process::OwnedProcess::spawn_tokio(
        command,
        crate::services::process_tree::ProcessKind::Searxng,
    )
    .await
    {
        Ok(child) => child,
        Err(_) => {
            return record(
                RuntimeCommandError::Spawn(stage),
                &OutputTail::new(),
                &OutputTail::new(),
            );
        }
    };
    let Some(root_pid) = child.id() else {
        crate::services::process_tree::terminate_tokio(
            &mut child,
            crate::services::process_tree::ProcessKind::Searxng,
        )
        .await;
        return record(
            RuntimeCommandError::Drain(stage),
            &OutputTail::new(),
            &OutputTail::new(),
        );
    };
    let (stdout, stderr) = match (child.stdout.take(), child.stderr.take()) {
        (Some(stdout), Some(stderr)) => (stdout, stderr),
        _ => {
            crate::services::process_tree::terminate_tokio(
                &mut child,
                crate::services::process_tree::ProcessKind::Searxng,
            )
            .await;
            return record(
                RuntimeCommandError::Drain(stage),
                &OutputTail::new(),
                &OutputTail::new(),
            );
        }
    };
    let mut drains = DrainTasks::start(stdout, stderr);
    let status = tokio::select! {
        result = child.wait() => result.map_err(|_| RuntimeCommandError::Drain(stage)),
        _ = cancel.cancelled() => Err(RuntimeCommandError::Cancelled(stage)),
        _ = tokio::time::sleep_until(deadline) => Err(RuntimeCommandError::Timeout(stage)),
    };
    match status {
        Ok(status) => {
            super::runtime_command_finish::after_parent(
                status,
                root_pid,
                &mut drains,
                stage,
                deadline,
                cancel,
            )
            .await
        }
        Err(error) => {
            super::runtime_command_finish::after_failure(&mut child, root_pid, &mut drains, error)
                .await
        }
    }
}

pub(super) fn record_output(
    error: RuntimeCommandError,
    output: DrainedOutput,
) -> Result<(), RuntimeCommandError> {
    record(error, &output.stdout, &output.stderr)
}

fn record(
    error: RuntimeCommandError,
    stdout: &OutputTail,
    stderr: &OutputTail,
) -> Result<(), RuntimeCommandError> {
    super::runtime_command_log::write(error, stdout.as_bytes(), stderr.as_bytes())
        .map_err(|_| RuntimeCommandError::Diagnostics(error.stage()))?;
    Err(error)
}
