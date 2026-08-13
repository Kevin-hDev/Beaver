use super::fire::{claim_once_in, run_wakeup_steps, OnceClaimOutcome, WakeupStepOutcome};
use super::Scheduler;
use crate::app_exit::AppExitCoordinator;
use crate::models::{
    ClgoConfig, ScheduledWakeup, WakeupRunErrorCode, WakeupRunStatus, WakeupSchedule,
};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;

fn once_wakeup(active: bool) -> ScheduledWakeup {
    ScheduledWakeup {
        id: "once".into(),
        name: "once".into(),
        model: "m".into(),
        provider: "ollama".into(),
        prompt: "p".into(),
        schedule: WakeupSchedule::Once {
            datetime: "2026-08-12T10:00".into(),
        },
        description: String::new(),
        active,
        paused_by_global: false,
        created_at: "2026-08-12T00:00:00Z".into(),
        agentic: false,
        working_dir: String::new(),
        skill_ids: Vec::new(),
        tool_names: Vec::new(),
    }
}

#[tokio::test]
async fn once_is_claimed_before_provider_dispatch() {
    let order = Arc::new(Mutex::new(Vec::new()));
    let claim_order = Arc::clone(&order);
    let dispatch_order = Arc::clone(&order);

    let result = run_wakeup_steps(
        true,
        &CancellationToken::new(),
        move || async move {
            claim_order.lock().await.push("claim");
            Ok::<_, String>(OnceClaimOutcome::Claimed)
        },
        move || async move {
            dispatch_order.lock().await.push("dispatch");
            Ok::<_, String>(())
        },
    )
    .await;

    assert_eq!(result, Ok(WakeupStepOutcome::Completed(())));
    assert_eq!(order.lock().await.as_slice(), &["claim", "dispatch"]);
}

#[tokio::test]
async fn inactive_once_is_silent_and_never_calls_the_provider() {
    let dispatched = Arc::new(AtomicBool::new(false));
    let provider_called = Arc::clone(&dispatched);

    let result = run_wakeup_steps(
        true,
        &CancellationToken::new(),
        || async { Ok::<_, String>(OnceClaimOutcome::Inactive) },
        move || async move {
            provider_called.store(true, Ordering::Release);
            Ok::<_, String>(())
        },
    )
    .await;

    assert_eq!(result, Ok(WakeupStepOutcome::SkippedInactive));
    assert!(!dispatched.load(Ordering::Acquire));
}

#[tokio::test]
async fn cancellation_after_claim_has_a_typed_journal_outcome() {
    let cancel = CancellationToken::new();
    let cancel_during_claim = cancel.clone();
    let dispatched = Arc::new(AtomicBool::new(false));
    let provider_called = Arc::clone(&dispatched);

    let result = run_wakeup_steps(
        true,
        &cancel,
        move || async move {
            cancel_during_claim.cancel();
            Ok::<_, String>(OnceClaimOutcome::Claimed)
        },
        move || async move {
            provider_called.store(true, Ordering::Release);
            Ok::<_, String>(())
        },
    )
    .await;

    assert_eq!(result, Ok(WakeupStepOutcome::Cancelled));
    assert!(!dispatched.load(Ordering::Acquire));
}

#[test]
fn once_claim_is_typed_and_mutates_only_an_active_once() {
    let mut config = ClgoConfig::default();
    config.scheduled_wakeups.push(once_wakeup(true));

    assert_eq!(
        claim_once_in(&mut config, "once"),
        OnceClaimOutcome::Claimed
    );
    assert!(!config.scheduled_wakeups[0].active);
    assert_eq!(
        claim_once_in(&mut config, "once"),
        OnceClaimOutcome::Inactive
    );
    assert_eq!(
        claim_once_in(&mut config, "missing"),
        OnceClaimOutcome::Inactive
    );
}

#[test]
fn cancelled_and_claim_error_entries_are_sanitized() {
    let scheduled_for = chrono::Local::now();
    let cancelled = super::log::cancelled_entry_for_test("once", scheduled_for);
    assert_eq!(cancelled.status, WakeupRunStatus::Cancelled);
    assert!(cancelled._legacy_error.is_none());
    assert!(cancelled.error_code.is_none());

    let error = super::log::error_entry_for_test(
        "once",
        scheduled_for,
        "Bearer secret at C:\\private\\config.json",
    );
    assert_eq!(error.status, WakeupRunStatus::Error);
    assert_eq!(error.error_code, Some(WakeupRunErrorCode::Failed));
    assert!(error._legacy_error.is_none());
}

#[tokio::test]
async fn scheduler_shutdown_waits_for_cleanup_and_refuses_late_wakeups() {
    let coordinator = AppExitCoordinator::initialize().expect("exit coordinator");
    let scheduler = Scheduler::for_test(coordinator.work_supervisor());
    let finished = Arc::new(AtomicBool::new(false));
    let task_finished = Arc::clone(&finished);
    let (started_tx, started_rx) = tokio::sync::oneshot::channel();
    scheduler
        .spawn_wakeup_for_test(move |cancel| async move {
            let _ = started_tx.send(());
            cancel.cancelled().await;
            tokio::time::sleep(Duration::from_millis(20)).await;
            task_finished.store(true, Ordering::Release);
        })
        .expect("scheduled wakeup");
    started_rx.await.unwrap();
    let running = scheduler.diagnostics();
    assert_eq!(running.wakeups.active, 1);
    assert_eq!(running.wakeups.high_water, 1);

    assert!(
        scheduler
            .stop_and_wait(Instant::now() + Duration::from_secs(1))
            .await
    );
    assert!(finished.load(Ordering::Acquire));
    assert_eq!(
        scheduler
            .spawn_wakeup_for_test(|_| async {})
            .expect_err("stopped scheduler must reject work")
            .public_code(),
        "service-shutting-down"
    );
    let stopped = scheduler.diagnostics();
    assert_eq!(stopped.wakeups.active, 0);
    assert_eq!(stopped.wakeups.closing_refusals, 1);
}

#[tokio::test]
async fn scheduler_shutdown_waits_for_its_persistent_loop() {
    let coordinator = AppExitCoordinator::initialize().expect("exit coordinator");
    let scheduler = Scheduler::for_test(coordinator.work_supervisor());
    let finished = Arc::new(AtomicBool::new(false));
    let loop_finished = Arc::clone(&finished);
    let (started_tx, started_rx) = tokio::sync::oneshot::channel();
    scheduler
        .spawn_loop_for_test(move |cancel| async move {
            let _ = started_tx.send(());
            cancel.cancelled().await;
            loop_finished.store(true, Ordering::Release);
        })
        .expect("scheduler loop");
    started_rx.await.unwrap();

    assert!(
        scheduler
            .stop_and_wait(Instant::now() + Duration::from_secs(1))
            .await
    );
    assert!(finished.load(Ordering::Acquire));
}
