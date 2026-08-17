use super::blocking;
use super::cleanup::{
    global_registry_is_empty, run_ordered, run_service_group, run_with_deadline, CleanupOutcome,
    StopFuture,
};
use std::sync::atomic::{AtomicUsize, Ordering};
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
async fn a_blocking_cleanup_panic_is_not_silently_ignored() {
    let outcome = run_with_deadline(
        Instant::now() + Duration::from_secs(1),
        blocking::execute(|| panic!("injected blocking cleanup panic")),
    )
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

#[tokio::test]
async fn one_service_timeout_does_not_skip_the_other_services() {
    let completed = Arc::new(AtomicUsize::new(0));
    let completed_by_neighbor = Arc::clone(&completed);
    let services: [(&'static str, StopFuture<'_>); 2] = [
        ("timed-out", Box::pin(async { false })),
        (
            "completed",
            Box::pin(async move {
                completed_by_neighbor.fetch_add(1, Ordering::SeqCst);
                true
            }),
        ),
    ];

    assert!(!run_service_group(services).await);
    assert_eq!(completed.load(Ordering::SeqCst), 1);
}

#[test]
fn the_global_registry_must_be_empty_after_service_cleanup() {
    assert!(global_registry_is_empty(0));
    assert!(!global_registry_is_empty(1));
}

mod ollama {
    #[test]
    fn cleanup_routes_ollama_through_manager_and_its_setup_deadline() {
        let source = include_str!("cleanup.rs");
        assert!(source.contains("stop_for_shutdown"));
        assert!(source.contains("ollama_setup_deadline"));
        assert!(source.contains("graceful_deadline"));
        assert!(!source.contains("ollama_lifecycle"));
        assert!(!source.contains("ollama_kill"));
    }
}
