use super::tool_bash_security::{blocked_reason, initialize_for_test};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

#[tokio::test]
async fn transient_spawn_failure_is_retried_and_success_is_reused() {
    let ready = tokio::sync::OnceCell::new();
    let attempts = Arc::new(AtomicUsize::new(0));

    let failed = initialize_for_test(&ready, |_| {
        attempts.fetch_add(1, Ordering::SeqCst);
        Err(std::io::Error::from(std::io::ErrorKind::WouldBlock))
    })
    .await;
    assert_eq!(failed, Err("Contrôle de commande indisponible.".to_string()));

    let success_attempts = attempts.clone();
    initialize_for_test(&ready, move |task| {
        success_attempts.fetch_add(1, Ordering::SeqCst);
        std::thread::Builder::new().spawn(task).map(|_| ())
    })
    .await
    .expect("retry succeeds");

    initialize_for_test(&ready, |_| panic!("successful initialization must be reused"))
        .await
        .expect("reuse succeeds");
    assert_eq!(attempts.load(Ordering::SeqCst), 2);
}

#[tokio::test]
async fn shared_boundary_distinguishes_allowed_and_blocked_commands() {
    assert_eq!(blocked_reason("printf safe").await, Ok(None));
    assert!(blocked_reason("sudo rm file.txt")
        .await
        .expect("command check")
        .is_some());
}
