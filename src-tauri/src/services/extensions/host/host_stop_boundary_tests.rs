use super::OperationFailure;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

#[tokio::test]
async fn incomplete_stop_never_polls_the_followup_operation() {
    let polled = Arc::new(AtomicBool::new(false));
    let observed = Arc::clone(&polled);

    let result = super::host_stop_boundary::after_confirmed_stop(
        false,
        OperationFailure::HostUnavailable,
        async move {
            observed.store(true, Ordering::Release);
            Ok(())
        },
    )
    .await;

    assert_eq!(result, Err(OperationFailure::HostUnavailable));
    assert!(!polled.load(Ordering::Acquire));
}
