use super::{execute_shell_managed, ShellExecutionContext};
use crate::app_exit::AppExitCoordinator;
use crate::services::agent_local::{agent_work_supervision::AgentWorkServices, tool_bash_registry};
use std::time::{Duration, Instant};
use tokio_util::sync::CancellationToken;

#[cfg(windows)]
const LONG_COMMAND: &str = "Start-Sleep -Seconds 30";
#[cfg(unix)]
const LONG_COMMAND: &str = "sleep 30";

#[cfg(any(windows, unix))]
#[tokio::test]
async fn service_shutdown_terminates_a_running_shell() {
    let coordinator = AppExitCoordinator::initialize().expect("exit coordinator");
    let services = AgentWorkServices::new(coordinator.work_supervisor());
    let directory = tempfile::tempdir().expect("temporary directory");
    let owner = uuid::Uuid::new_v4().to_string();

    let output = execute_shell_managed(
        LONG_COMMAND,
        directory.path(),
        ShellExecutionContext {
            owner_session_id: &owner,
            hard_timeout_secs: None,
            yield_time_ms: Some(250),
            cancel: CancellationToken::new(),
            progress: None,
            work: services.shells(),
        },
    )
    .await
    .expect("long shell starts");
    assert!(output.running);
    let process_id = output
        .stdout
        .split("session_id=")
        .nth(1)
        .and_then(|tail| tail.split(',').next())
        .expect("process id")
        .to_string();

    assert!(
        services
            .shells()
            .stop_and_wait(Instant::now() + Duration::from_secs(5))
            .await
    );
    assert_eq!(services.shells().diagnostics().active, 0);
    tool_bash_registry::remove(&process_id);
}
