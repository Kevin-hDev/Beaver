use super::owned_probe::{self, ProbeSpec, MAX_PROBE_STDOUT_BYTES};
use crate::app_exit::AppExitCoordinator;
use crate::services::work_registry::ServiceWorkSupervisor;
use std::time::Duration;

fn python_spec(script: &str) -> ProbeSpec {
    ProbeSpec::new(crate::services::test_runtime::python().expect("test Python"))
        .args(["-c", script])
}

#[tokio::test]
async fn cancellation_reaps_the_owned_probe_process() {
    let coordinator = AppExitCoordinator::initialize().expect("exit coordinator");
    let supervisor = ServiceWorkSupervisor::<1>::new(coordinator.work_supervisor());
    let admission = supervisor.try_admit().expect("probe admission");
    let cancel = admission.cancellation();
    let (started_tx, started_rx) = tokio::sync::oneshot::channel();
    let probe = tokio::spawn(async move {
        owned_probe::run_for_test(
            python_spec("import time; time.sleep(2)"),
            &cancel,
            started_tx,
        )
        .await
    });
    let pid = started_rx.await.expect("probe pid");

    coordinator.close_work_admission_for_test();
    assert!(tokio::time::timeout(Duration::from_secs(3), probe)
        .await
        .expect("probe cancellation deadline")
        .expect("probe task")
        .is_none());
    assert!(
        wait_until_process_is_gone(pid).await,
        "cancelled GPU probe survived"
    );
    drop(admission);
}

#[tokio::test]
async fn excessive_probe_output_is_truncated_at_the_single_bound() {
    let coordinator = AppExitCoordinator::initialize().expect("exit coordinator");
    let supervisor = ServiceWorkSupervisor::<1>::new(coordinator.work_supervisor());
    let admission = supervisor.try_admit().expect("probe admission");
    let cancel = admission.cancellation();
    let output = owned_probe::run(
        python_spec(&format!(
            "import sys; sys.stdout.write('x' * {})",
            MAX_PROBE_STDOUT_BYTES * 2
        )),
        &cancel,
    )
    .await
    .expect("probe output");

    assert_eq!(output.stdout.len(), MAX_PROBE_STDOUT_BYTES);
    assert!(output.truncated);
}

#[tokio::test]
async fn operational_timeout_reaps_the_probe_process() {
    let coordinator = AppExitCoordinator::initialize().expect("exit coordinator");
    let supervisor = ServiceWorkSupervisor::<1>::new(coordinator.work_supervisor());
    let admission = supervisor.try_admit().expect("probe admission");
    let cancel = admission.cancellation();
    let (started_tx, started_rx) = tokio::sync::oneshot::channel();
    let started = std::time::Instant::now();
    let probe = tokio::spawn(async move {
        owned_probe::run_for_test(
            python_spec("import time; time.sleep(30)"),
            &cancel,
            started_tx,
        )
        .await
    });
    let pid = started_rx.await.expect("probe pid");

    assert!(probe.await.expect("probe task").is_none());
    assert!(started.elapsed() < Duration::from_secs(5));
    assert!(
        wait_until_process_is_gone(pid).await,
        "timed out GPU probe survived"
    );
    drop(admission);
}

async fn wait_until_process_is_gone(pid: u32) -> bool {
    let deadline = std::time::Instant::now() + Duration::from_secs(2);
    loop {
        let mut processes = sysinfo::System::new();
        processes.refresh_processes(sysinfo::ProcessesToUpdate::All, true);
        if processes.process(sysinfo::Pid::from_u32(pid)).is_none() {
            return true;
        }
        if std::time::Instant::now() >= deadline {
            return false;
        }
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
}

#[cfg(windows)]
#[tokio::test]
async fn powershell_probe_is_registered_in_owned_process_authority() {
    let coordinator = AppExitCoordinator::initialize().expect("exit coordinator");
    let supervisor = ServiceWorkSupervisor::<1>::new(coordinator.work_supervisor());
    let admission = supervisor.try_admit().expect("probe admission");
    let cancel = admission.cancellation();
    let (started_tx, started_rx) = tokio::sync::oneshot::channel();
    let probe = tokio::spawn(async move {
        owned_probe::run_for_test(
            ProbeSpec::new("powershell").args([
                "-NoProfile",
                "-NonInteractive",
                "-Command",
                "Start-Sleep -Seconds 2",
            ]),
            &cancel,
            started_tx,
        )
        .await
    });
    let pid = started_rx.await.expect("PowerShell pid");

    assert!(crate::services::owned_process::OwnedProcess::is_confined_for_test(pid));
    coordinator.close_work_admission_for_test();
    let _ = probe.await;
    drop(admission);
}
