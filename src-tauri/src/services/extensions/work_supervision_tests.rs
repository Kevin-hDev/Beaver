use super::work_supervision::{
    ExtensionWorkAdmissionError, ExtensionWorkServices, MAX_EXTENSION_CORE_CALLS,
    MAX_EXTENSION_OPERATIONS,
};
use crate::app_exit::AppExitCoordinator;
use crate::services::work_registry::ServiceWorkPhase;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

fn extension_work() -> ExtensionWorkServices {
    let coordinator = AppExitCoordinator::initialize().expect("exit coordinator");
    ExtensionWorkServices::new(coordinator.work_supervisor())
}

#[test]
fn operation_and_core_call_registries_have_fixed_capacities() {
    let work = extension_work();
    let operations = (0..MAX_EXTENSION_OPERATIONS)
        .map(|_| work.try_admit_operation().expect("operation slot"))
        .collect::<Vec<_>>();
    assert_eq!(
        work.try_admit_operation()
            .expect_err("operation capacity must be bounded"),
        ExtensionWorkAdmissionError::Busy
    );
    assert_eq!(work.operation_diagnostics().saturation_refusals, 1);
    drop(operations);

    let core_calls = (0..MAX_EXTENSION_CORE_CALLS)
        .map(|_| work.try_admit_core_call().expect("core call slot"))
        .collect::<Vec<_>>();
    assert_eq!(
        work.try_admit_core_call()
            .expect_err("core call capacity must be bounded"),
        ExtensionWorkAdmissionError::Busy
    );
    assert_eq!(work.core_call_diagnostics().saturation_refusals, 1);
    drop(core_calls);

    for _ in 0..(MAX_EXTENSION_OPERATIONS * 4) {
        drop(
            work.try_admit_operation()
                .expect("released slot is reusable"),
        );
    }
}

#[tokio::test]
async fn stop_cancels_and_awaits_reader_operation_and_core_call() {
    let work = extension_work();
    let completed = Arc::new(AtomicUsize::new(0));

    let reader_completed = Arc::clone(&completed);
    work.try_admit_reader()
        .expect("reader admission")
        .spawn(move |cancel| async move {
            cancel.cancelled().await;
            reader_completed.fetch_add(1, Ordering::SeqCst);
        })
        .expect("supervised extension reader starts");
    let operation_completed = Arc::clone(&completed);
    work.spawn_operation(move |cancel| async move {
        cancel.cancelled().await;
        operation_completed.fetch_add(1, Ordering::SeqCst);
    })
    .expect("supervised extension operation starts");
    let core_call_completed = Arc::clone(&completed);
    work.spawn_core_call(move |cancel| async move {
        cancel.cancelled().await;
        core_call_completed.fetch_add(1, Ordering::SeqCst);
    })
    .expect("supervised extension core call starts");

    work.begin_closing();
    assert_eq!(
        work.try_admit_operation()
            .expect_err("closing refuses extension restart"),
        ExtensionWorkAdmissionError::ShuttingDown
    );
    assert!(
        work.stop_and_wait(Instant::now() + Duration::from_secs(1))
            .await
    );
    assert_eq!(completed.load(Ordering::SeqCst), 3);
    assert_eq!(work.reader_phase(), ServiceWorkPhase::Closed);
    assert_eq!(work.operation_phase(), ServiceWorkPhase::Closed);
    assert_eq!(work.core_call_phase(), ServiceWorkPhase::Closed);
}

#[tokio::test]
async fn stop_is_idempotent_and_permanently_refuses_restart() {
    let work = extension_work();
    let deadline = Instant::now() + Duration::from_secs(1);

    assert!(work.stop_and_wait(deadline).await);
    assert!(work.stop_and_wait(deadline).await);
    assert_eq!(
        work.spawn_operation(|_| async {}),
        Err(ExtensionWorkAdmissionError::ShuttingDown)
    );
}
