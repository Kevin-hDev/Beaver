use super::runtime::handle_due_admission;
use super::work_supervision::SchedulerWorkServices;
use crate::app_exit::AppExitCoordinator;
use crate::services::work_registry::ServiceWorkAdmissionError;
use chrono::Local;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

#[tokio::test]
async fn saturated_wakeup_work_records_capacity_and_keeps_the_loop_alive() {
    let coordinator = AppExitCoordinator::initialize().expect("exit coordinator");
    let work = SchedulerWorkServices::new(coordinator.work_supervisor()).wakeups();
    let refusal = loop {
        match work.spawn(|cancel| async move { cancel.cancelled().await }) {
            Ok(()) => {}
            Err(error) => break error,
        }
    };
    let recorded = Arc::new(Mutex::new(Vec::new()));
    let recorded_by_writer = Arc::clone(&recorded);

    let keep_running = handle_due_admission(
        Err(refusal),
        "capacity-wakeup".into(),
        Local::now(),
        move |_, _, error| async move {
            recorded_by_writer.lock().unwrap().push(error);
            Ok(())
        },
    )
    .await;

    assert!(keep_running);
    assert_eq!(
        recorded.lock().unwrap().as_slice(),
        &[ServiceWorkAdmissionError::Capacity]
    );
    work.begin_closing();
    assert!(
        work.stop_and_wait(Instant::now() + Duration::from_secs(1))
            .await
    );
}

#[tokio::test]
async fn closed_wakeup_work_records_closing_and_stops_the_due_loop() {
    let coordinator = AppExitCoordinator::initialize().expect("exit coordinator");
    let work = SchedulerWorkServices::new(coordinator.work_supervisor()).wakeups();
    work.begin_closing();
    let refusal = work.spawn(|_| async {}).unwrap_err();
    let recorded = Arc::new(Mutex::new(Vec::new()));
    let recorded_by_writer = Arc::clone(&recorded);

    let keep_running = handle_due_admission(
        Err(refusal),
        "closing-wakeup".into(),
        Local::now(),
        move |_, _, error| async move {
            recorded_by_writer.lock().unwrap().push(error);
            Ok(())
        },
    )
    .await;

    assert!(!keep_running);
    assert_eq!(
        recorded.lock().unwrap().as_slice(),
        &[ServiceWorkAdmissionError::Closing]
    );
}
