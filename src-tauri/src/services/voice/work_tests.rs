use super::errors::VoiceErrorCode;
use super::runtime::new_voice_runtime;
use super::types::VoicePhase;
use std::sync::{Arc, Barrier};
use std::time::{Duration, Instant};
use tokio::sync::Notify;

fn runtime() -> super::runtime::VoiceRuntime {
    let exit = crate::app_exit::AppExitCoordinator::initialize().expect("exit coordinator");
    new_voice_runtime(exit.work_supervisor())
}

#[tokio::test]
async fn detached_wait_does_not_release_the_native_owner() {
    let runtime = runtime();
    let started = Arc::new(Notify::new());
    let release = Arc::new(Barrier::new(2));
    let started_in_work = Arc::clone(&started);
    let release_in_work = Arc::clone(&release);
    let waiting = runtime
        .spawn_blocking(move |context| {
            assert!(!context.is_cancelled());
            context
                .transition(VoicePhase::Listening)
                .expect("listening transition");
            started_in_work.notify_one();
            release_in_work.wait();
        })
        .expect("voice work admitted");
    started.notified().await;
    drop(waiting);

    assert_eq!(runtime.phase(), VoicePhase::Listening);
    assert_eq!(runtime.active_work(), 1);
    assert_eq!(
        runtime
            .try_reserve()
            .err()
            .expect("second reservation")
            .code(),
        VoiceErrorCode::Busy
    );
    assert!(
        !runtime
            .stop_and_wait(Instant::now() + Duration::from_millis(20))
            .await
    );
    assert_eq!(runtime.active_work(), 1);
    release.wait();
    runtime.wait_until_inactive_for_test().await;
    assert_eq!(runtime.active_work(), 0);
}

#[test]
fn a_direct_reservation_owns_the_state_until_drop() {
    let runtime = runtime();
    let reservation = runtime.try_reserve().expect("reservation");
    reservation
        .context()
        .transition(VoicePhase::Transcribing)
        .expect("transcribing transition");
    assert_eq!(runtime.phase(), VoicePhase::Transcribing);
    drop(reservation);
    assert_eq!(runtime.phase(), VoicePhase::Idle);
    assert_eq!(runtime.active_work(), 0);
}

#[tokio::test]
async fn error_and_panic_release_the_owner_once() {
    let error_runtime = runtime();
    let result = error_runtime
        .spawn_blocking(|_| Err::<(), _>("expected"))
        .expect("error work")
        .await
        .expect("error result delivered");
    assert!(result.is_err());
    error_runtime.wait_until_inactive_for_test().await;

    let panic_runtime = runtime();
    let completion = panic_runtime
        .spawn_blocking(|_| -> () { panic!("expected test panic") })
        .expect("panic work");
    assert!(completion.await.is_err());
    panic_runtime.wait_until_inactive_for_test().await;
    assert_eq!(panic_runtime.active_work(), 0);
}
