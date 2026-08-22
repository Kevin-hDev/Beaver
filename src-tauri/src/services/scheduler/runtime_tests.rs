use super::due::{missed_occurrences, reconciliation_cutoff, ReconciliationMode};
use super::in_flight::{InFlightReservationError, InFlightWakeups};
use super::runtime_decisions::{handle_due_admission, persist_once_missed_decision};
use super::work_supervision::SchedulerWorkServices;
use crate::app_exit::AppExitCoordinator;
use crate::services::work_registry::ServiceWorkAdmissionError;
use chrono::{Local, TimeZone};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

fn daily_wakeup(
    id: &str,
    scheduled_for: chrono::DateTime<Local>,
) -> crate::models::ScheduledWakeup {
    crate::models::ScheduledWakeup {
        id: id.into(),
        name: id.into(),
        model: "m".into(),
        provider: "ollama".into(),
        prompt: "p".into(),
        schedule: crate::models::WakeupSchedule::Daily {
            time: scheduled_for.format("%H:%M").to_string(),
        },
        description: String::new(),
        project_id: None,
        active: true,
        paused_by_global: false,
        created_at: "2026-05-17T00:00:00Z".into(),
    }
}

#[test]
fn in_flight_occurrence_blocks_missed_decision_until_terminal_result() {
    let scheduled_for = Local
        .with_ymd_and_hms(2026, 8, 13, 8, 0, 0)
        .single()
        .unwrap();
    let wakeup = daily_wakeup("daily", scheduled_for);
    let in_flight = InFlightWakeups::default();
    let guard = in_flight.reserve("daily", scheduled_for).unwrap();

    let blocked = in_flight.partition(vec![(wakeup.clone(), scheduled_for)]);
    assert!(blocked.decidable.is_empty());
    assert!(blocked.has_in_flight);

    drop(guard);
    let released = in_flight.partition(vec![(wakeup, scheduled_for)]);
    assert_eq!(released.decidable.len(), 1);
    assert!(!released.has_in_flight);
}

#[test]
fn in_flight_registry_rejects_duplicates_and_a_sixty_fifth_occurrence() {
    let scheduled_for = Local
        .with_ymd_and_hms(2026, 8, 13, 8, 0, 0)
        .single()
        .unwrap();
    let in_flight = InFlightWakeups::default();
    let first = in_flight.reserve("wakeup-0", scheduled_for).unwrap();
    assert!(matches!(
        in_flight.reserve("wakeup-0", scheduled_for),
        Err(InFlightReservationError::Duplicate)
    ));

    let mut guards = vec![first];
    for index in 1..64 {
        guards.push(
            in_flight
                .reserve(&format!("wakeup-{index}"), scheduled_for)
                .unwrap(),
        );
    }
    assert!(matches!(
        in_flight.reserve("wakeup-64", scheduled_for),
        Err(InFlightReservationError::Capacity)
    ));
}

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

    assert!(keep_running.keep_running);
    assert!(keep_running.decision_persisted);
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

    assert!(!keep_running.keep_running);
    assert!(keep_running.decision_persisted);
    assert_eq!(
        recorded.lock().unwrap().as_slice(),
        &[ServiceWorkAdmissionError::Closing]
    );
}

#[tokio::test]
async fn failed_refusal_write_keeps_occurrence_reconcilable() {
    let scheduled_for = Local::now() - chrono::Duration::minutes(6);
    let outcome = handle_due_admission(
        Err(ServiceWorkAdmissionError::Capacity),
        "retry-wakeup".into(),
        scheduled_for,
        |_, _, _| async { Err("injected-log-failure".to_string()) },
    )
    .await;

    assert!(outcome.keep_running);
    assert!(!outcome.decision_persisted);
    let cutoff = reconciliation_cutoff(Local::now(), ReconciliationMode::Running);
    let wakeup = daily_wakeup("retry-wakeup", scheduled_for);
    assert_eq!(
        missed_occurrences(
            &[wakeup],
            scheduled_for - chrono::Duration::minutes(1),
            cutoff
        )
        .len(),
        1
    );
}

#[tokio::test]
async fn once_wakeup_is_not_claimed_before_its_missed_decision_is_durable() {
    let calls = Arc::new(Mutex::new(Vec::new()));
    let recorded = Arc::clone(&calls);
    let claimed = Arc::clone(&calls);

    let result = persist_once_missed_decision(
        move || async move {
            recorded.lock().unwrap().push("record");
            Err("injected-log-failure".to_string())
        },
        move || {
            claimed.lock().unwrap().push("claim");
            Ok(())
        },
    )
    .await;

    assert!(result.is_err());
    assert_eq!(calls.lock().unwrap().as_slice(), &["record"]);
}
