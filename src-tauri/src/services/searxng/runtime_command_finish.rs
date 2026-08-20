use std::process::ExitStatus;

use tokio::time::Instant;

use crate::services::work_registry::ServiceWorkCancellation;

use super::runtime_command::{record_output, RuntimeCommandError, RuntimeStage};
use super::runtime_command_drain::{DrainTasks, DrainWait, DrainedOutput};

pub(super) async fn after_parent(
    status: ExitStatus,
    root_pid: u32,
    drains: &mut DrainTasks,
    stage: RuntimeStage,
    deadline: Instant,
    cancel: &ServiceWorkCancellation,
) -> Result<(), RuntimeCommandError> {
    match drains.wait(deadline, cancel).await {
        DrainWait::Ready => {
            let output = drains.take_output();
            if output.failed {
                record_output(RuntimeCommandError::Drain(stage), output)
            } else {
                finish_status(status, stage, output)
            }
        }
        DrainWait::Failed => {
            stop_after_parent(RuntimeCommandError::Drain(stage), root_pid, drains).await
        }
        DrainWait::Cancelled => {
            stop_after_parent(RuntimeCommandError::Cancelled(stage), root_pid, drains).await
        }
        DrainWait::TimedOut => {
            stop_after_parent(RuntimeCommandError::Timeout(stage), root_pid, drains).await
        }
    }
}

pub(super) async fn after_failure(
    child: &mut tokio::process::Child,
    root_pid: u32,
    drains: &mut DrainTasks,
    error: RuntimeCommandError,
) -> Result<(), RuntimeCommandError> {
    crate::services::process_tree::terminate_tokio(
        child,
        crate::services::process_tree::ProcessKind::Searxng,
    )
    .await;
    stop_inherited_pipes(root_pid).await;
    record_output(error, drains.abort_and_collect().await)
}

async fn stop_after_parent(
    error: RuntimeCommandError,
    root_pid: u32,
    drains: &mut DrainTasks,
) -> Result<(), RuntimeCommandError> {
    stop_inherited_pipes(root_pid).await;
    record_output(error, drains.abort_and_collect().await)
}

fn finish_status(
    status: ExitStatus,
    stage: RuntimeStage,
    output: DrainedOutput,
) -> Result<(), RuntimeCommandError> {
    if status.success() {
        Ok(())
    } else {
        record_output(RuntimeCommandError::NonZero(stage, status.code()), output)
    }
}

async fn stop_inherited_pipes(root_pid: u32) {
    let _ = tokio::task::spawn_blocking(move || {
        crate::services::process_tree::kill_pipe_holders_after_parent_exit(
            root_pid,
            crate::services::process_tree::ProcessKind::Searxng,
        );
    })
    .await;
}
