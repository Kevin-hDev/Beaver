use super::registry::{AdmissionError, AdmissionRegistry, REGISTRY_CAPACITY};
use std::future::pending;
use std::time::{Duration, Instant};

#[test]
fn registry_is_bounded_and_fails_closed_at_capacity() {
    let registry = AdmissionRegistry::new();
    let admissions = (0..REGISTRY_CAPACITY)
        .map(|_| registry.try_admit().expect("available slot"))
        .collect::<Vec<_>>();

    assert_eq!(registry.active_count(), REGISTRY_CAPACITY);
    assert_eq!(registry.try_admit().unwrap_err(), AdmissionError::Capacity);
    drop(admissions);
    assert_eq!(registry.active_count(), 0);
}

#[test]
fn close_rejects_new_work_and_cancels_admitted_work() {
    let registry = AdmissionRegistry::new();
    let admission = registry.try_admit().expect("admission");
    let child_cancel = admission.cancellation_token();

    assert!(registry.close());
    assert!(!registry.close());
    assert!(child_cancel.is_cancelled());
    assert_eq!(registry.try_admit().unwrap_err(), AdmissionError::Closing);
    assert_eq!(AdmissionError::Closing.public_code(), "app-shutting-down");
}

#[test]
fn stale_generation_cannot_release_a_reused_slot() {
    let registry = AdmissionRegistry::new();
    let first = registry.try_admit().expect("first admission");
    let stale = first.key_for_test();
    drop(first);
    let second = registry.try_admit().expect("second admission");

    assert_ne!(stale, second.key_for_test());
    assert!(!registry.release_key_for_test(stale));
    assert_eq!(registry.active_count(), 1);
    drop(second);
    assert_eq!(registry.active_count(), 0);
}

#[tokio::test]
async fn normal_completion_releases_the_slot() {
    let registry = AdmissionRegistry::new();
    let admission = registry.try_admit().expect("admission");
    let value = admission.run(async { 42 }).await;

    assert_eq!(value, 42);
    assert_eq!(registry.active_count(), 0);
}

#[tokio::test]
async fn panic_and_abort_release_their_slots() {
    let registry = AdmissionRegistry::new();
    let panic_admission = registry.try_admit().expect("panic admission");
    let panic_task = tokio::spawn(panic_admission.run(async {
        panic!("injected task panic");
    }));
    assert!(panic_task.await.expect_err("task must panic").is_panic());

    let abort_admission = registry.try_admit().expect("abort admission");
    let abort_task = tokio::spawn(abort_admission.run(pending::<()>()));
    tokio::task::yield_now().await;
    abort_task.abort();
    assert!(abort_task
        .await
        .expect_err("task must abort")
        .is_cancelled());
    assert_eq!(registry.active_count(), 0);
}

#[tokio::test]
async fn wait_uses_the_supplied_absolute_deadline() {
    let registry = AdmissionRegistry::new();
    let admission = registry.try_admit().expect("admission");
    let release = tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(10)).await;
        drop(admission);
    });
    assert!(
        registry
            .wait_empty_until(Instant::now() + Duration::from_secs(1))
            .await
    );
    release.await.expect("release task");

    let _held = registry.try_admit().expect("held admission");
    assert!(
        !registry
            .wait_empty_until(Instant::now() + Duration::from_millis(10))
            .await
    );
}

#[test]
fn close_and_admission_are_linearized() {
    for _ in 0..256 {
        let registry = AdmissionRegistry::new();
        let closer = registry.clone();
        let requester = registry.clone();
        let close = std::thread::spawn(move || closer.close());
        let admission = std::thread::spawn(move || requester.try_admit());
        close.join().expect("close thread");
        let result = admission.join().expect("admission thread");

        if let Ok(admission) = result {
            assert!(admission.cancellation_token().is_cancelled());
        }
        assert_eq!(registry.try_admit().unwrap_err(), AdmissionError::Closing);
    }
}
