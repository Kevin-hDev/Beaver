use super::policy::ShutdownPolicy;
use super::ultimate::{RawExitActions, UltimateExit};
use super::{AppExitCoordinator, AppWorkAdmissionError};
use crate::services::browser::CefShutdownBarrier;
use std::time::{Duration, Instant};

fn coordinator() -> AppExitCoordinator {
    let policy = ShutdownPolicy::new(
        Duration::from_millis(100),
        Duration::from_millis(200),
        Duration::from_millis(300),
        Duration::from_secs(1),
    )
    .expect("supervisor policy");
    let ultimate =
        UltimateExit::initialize_for_test(Instant::now(), RawExitActions::testing(|_| {}, |_| {}))
            .expect("ultimate thread");
    AppExitCoordinator::from_parts_for_test(policy, ultimate)
}

#[test]
fn coordinator_supervisor_cancels_admitted_work_and_refuses_late_starts() {
    let coordinator = coordinator();
    let supervisor = coordinator.work_supervisor();
    let admission = supervisor.try_admit().expect("tracked app work");
    let cancellation = admission.cancellation_token();

    coordinator.begin_with_cef_close(0, |_, _, _| CefShutdownBarrier::Drained);

    assert!(cancellation.is_cancelled());
    let error = supervisor
        .try_admit()
        .expect_err("late work must be refused");
    assert_eq!(error, AppWorkAdmissionError::Closing);
    assert_eq!(error.public_code(), "app-shutting-down");
}

#[test]
fn coordinator_supervisor_reuses_released_slots_without_false_saturation() {
    let coordinator = coordinator();
    let supervisor = coordinator.work_supervisor();
    let admissions = (0..super::registry::REGISTRY_CAPACITY)
        .map(|_| supervisor.try_admit().expect("available app work slot"))
        .collect::<Vec<_>>();

    let error = supervisor
        .try_admit()
        .expect_err("capacity must remain bounded");
    assert_eq!(error, AppWorkAdmissionError::Capacity);
    assert_eq!(error.public_code(), "app-work-capacity-reached");
    drop(admissions);

    for _ in 0..(super::registry::REGISTRY_CAPACITY * 4) {
        drop(
            supervisor
                .try_admit()
                .expect("released slot must be reusable"),
        );
    }
}
