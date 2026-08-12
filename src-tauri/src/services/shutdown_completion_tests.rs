use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

#[tokio::test]
async fn domain_work_is_awaited_after_process_failure() {
    let awaited = Arc::new(AtomicBool::new(false));
    let observed = Arc::clone(&awaited);

    let stopped = super::shutdown_completion::combine_with_work(false, async move {
        observed.store(true, Ordering::Release);
        true
    })
    .await;

    assert!(!stopped);
    assert!(awaited.load(Ordering::Acquire));
}
