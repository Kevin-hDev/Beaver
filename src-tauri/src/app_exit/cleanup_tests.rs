use super::blocking;
use super::cleanup::{run_ordered, run_with_deadline, CleanupOutcome};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

#[tokio::test]
async fn a_real_blocking_call_cannot_block_the_async_deadline() {
    let started = Instant::now();
    let operation = blocking::execute(|| {
        std::thread::sleep(Duration::from_millis(150));
    });
    let outcome = run_with_deadline(Instant::now() + Duration::from_millis(20), operation).await;

    assert_eq!(outcome, CleanupOutcome::TimedOut);
    assert!(started.elapsed() < Duration::from_millis(100));
}

#[tokio::test]
async fn an_expired_absolute_deadline_does_not_create_a_second_wait() {
    let started = Instant::now();
    let outcome = run_with_deadline(
        Instant::now() - Duration::from_millis(1),
        std::future::pending::<()>(),
    )
    .await;

    assert_eq!(outcome, CleanupOutcome::TimedOut);
    assert!(started.elapsed() < Duration::from_millis(20));
}

#[tokio::test]
async fn a_cleanup_panic_is_contained() {
    let outcome = run_with_deadline(Instant::now() + Duration::from_secs(1), async {
        panic!("injected cleanup panic");
    })
    .await;

    assert_eq!(outcome, CleanupOutcome::Panicked);
}

#[tokio::test]
async fn ollama_phase_runs_after_the_other_services() {
    let order = Arc::new(Mutex::new(Vec::with_capacity(2)));
    let services_order = Arc::clone(&order);
    let ollama_order = Arc::clone(&order);
    run_ordered(
        async move {
            services_order
                .lock()
                .expect("services order")
                .push("services");
        },
        async move {
            ollama_order.lock().expect("ollama order").push("ollama");
        },
    )
    .await;

    assert_eq!(*order.lock().expect("final order"), ["services", "ollama"]);
}

#[tokio::test]
async fn completed_cleanup_reports_success() {
    assert_eq!(
        run_with_deadline(Instant::now() + Duration::from_secs(1), async {}).await,
        CleanupOutcome::Completed
    );
}
