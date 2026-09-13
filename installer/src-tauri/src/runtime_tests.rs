use super::contract::{InstallerOutcome, InstallerPhase, ProgressMode};
use super::runtime::{InstallerRuntime, PlatformKind};

fn new_runtime(platform: PlatformKind) -> InstallerRuntime {
    InstallerRuntime::new("1.2.3", "/Applications", None, false, platform)
}

#[test]
fn publishes_the_five_steps_in_order_and_completes() {
    let runtime = new_runtime(PlatformKind::Macos);
    let operation = runtime.begin().unwrap();
    let phases = [
        InstallerPhase::Checking,
        InstallerPhase::Downloading,
        InstallerPhase::Verifying,
        InstallerPhase::Installing,
        InstallerPhase::Finishing,
    ];
    assert_eq!(runtime.snapshot().snapshot.phase, phases[0]);
    runtime.downloading(10).unwrap();
    runtime.verifying().unwrap();
    runtime.installing(true).unwrap();
    runtime.begin_swap().unwrap();
    runtime.finishing().unwrap();
    let completed = runtime
        .succeed(operation, InstallerOutcome::Installed)
        .unwrap();
    assert_eq!(completed.snapshot.phase, InstallerPhase::Completed);
    assert_eq!(completed.snapshot.step_index, 5);
    assert_eq!(completed.snapshot.step_count, 5);
    assert_eq!(completed.completed_step_durations_ms.len(), 5);
    assert_eq!(
        completed.snapshot.outcome,
        Some(InstallerOutcome::Installed)
    );
}

#[test]
fn allows_only_one_active_operation() {
    let runtime = new_runtime(PlatformKind::Macos);
    let operation = runtime.begin().unwrap();
    assert!(runtime.begin().is_err());
    runtime.fail(operation, "installer.errors.install").unwrap();
    assert!(runtime.begin().is_ok());
}

#[test]
fn active_operation_blocks_application_shutdown() {
    let runtime = new_runtime(PlatformKind::Macos);
    assert!(!runtime.operation_active());
    let operation = runtime.begin().unwrap();
    assert!(runtime.operation_active());
    runtime.fail(operation, "installer.errors.install").unwrap();
    assert!(!runtime.operation_active());
}

#[test]
fn macos_can_cancel_staging_but_not_after_begin_swap() {
    let runtime = new_runtime(PlatformKind::Macos);
    let operation = runtime.begin().unwrap();
    runtime.downloading(1).unwrap();
    runtime.verifying().unwrap();
    runtime.installing(true).unwrap();
    assert!(runtime.snapshot().snapshot.can_cancel);
    assert!(runtime.cancel());
    assert!(operation.is_cancelled());
    runtime.cancelled(operation).unwrap();

    let operation = runtime.begin().unwrap();
    runtime.downloading(1).unwrap();
    runtime.verifying().unwrap();
    runtime.installing(true).unwrap();
    let event = runtime.begin_swap().unwrap();
    assert!(!event.snapshot.can_cancel);
    assert!(!runtime.cancel());
    assert!(!operation.is_cancelled());
}

#[test]
fn windows_disables_cancel_before_nsis_and_cancel_is_idempotent() {
    let runtime = new_runtime(PlatformKind::Windows);
    let operation = runtime.begin().unwrap();
    runtime.downloading(1).unwrap();
    runtime.verifying().unwrap();
    let event = runtime.installing(false).unwrap();
    assert!(!event.snapshot.can_cancel);
    assert!(!runtime.cancel());
    assert!(!runtime.cancel());
    assert!(!operation.is_cancelled());
}

#[test]
fn failed_and_cancelled_operations_can_retry() {
    let runtime = new_runtime(PlatformKind::Macos);
    let operation = runtime.begin().unwrap();
    runtime
        .fail(operation, "installer.errors.download")
        .unwrap();
    assert_eq!(runtime.snapshot().snapshot.phase, InstallerPhase::Failed);
    assert!(!runtime.begin().unwrap().is_cancelled());

    let runtime = new_runtime(PlatformKind::Macos);
    let operation = runtime.begin().unwrap();
    assert!(runtime.cancel());
    runtime.cancelled(operation).unwrap();
    assert_eq!(runtime.snapshot().snapshot.phase, InstallerPhase::Cancelled);
    assert!(runtime.begin().is_ok());
}

#[test]
fn download_progress_is_bounded() {
    let runtime = new_runtime(PlatformKind::Macos);
    runtime.begin().unwrap();
    let event = runtime.downloading(200).unwrap();
    assert_eq!(event.snapshot.progress_mode, ProgressMode::Determinate);
    assert_eq!(event.snapshot.percent, Some(100));
    assert_eq!(event.log_key.as_deref(), Some("installer.log.downloading"));
    assert_eq!(runtime.downloading(99).unwrap().log_key, None);
}
