use super::policy::{ShutdownPolicy, ShutdownTimeline};
use super::ultimate::{RawExitActions, UltimateExit};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, Instant};

fn test_timeline(origin: Instant) -> ShutdownTimeline {
    let policy = ShutdownPolicy::new(
        Duration::from_millis(5),
        Duration::from_millis(10),
        Duration::from_millis(20),
        Duration::from_millis(30),
    )
    .expect("test policy");
    ShutdownTimeline::from_origin(origin, policy)
}

fn wait_for_count(counter: &AtomicUsize, expected: usize) {
    let deadline = Instant::now() + Duration::from_secs(1);
    while counter.load(Ordering::Acquire) != expected {
        assert!(Instant::now() < deadline, "ultimate action deadline");
        std::thread::yield_now();
    }
}

#[test]
fn creation_failure_is_reported_before_any_effect() {
    let effects = AtomicUsize::new(0);
    let result = UltimateExit::initialize_with(
        Instant::now(),
        RawExitActions::testing(|_| {}, |_| {}),
        |_| Err(std::io::Error::other("injected spawn failure")),
    );
    if result.is_ok() {
        effects.fetch_add(1, Ordering::Release);
    }

    assert!(result.is_err());
    assert_eq!(effects.load(Ordering::Acquire), 0);
}

#[test]
fn arm_uses_one_absolute_deadline_and_fires_once() {
    let calls = Arc::new(AtomicUsize::new(0));
    let observed_code = Arc::new(AtomicUsize::new(0));
    let action_calls = Arc::clone(&calls);
    let action_code = Arc::clone(&observed_code);
    let origin = Instant::now();
    let mut ultimate = UltimateExit::initialize_for_test(
        origin,
        RawExitActions::testing(
            move |code| {
                action_code.store(code as usize, Ordering::Release);
                action_calls.fetch_add(1, Ordering::AcqRel);
            },
            |_| {},
        ),
    )
    .expect("ultimate thread");
    let timeline = test_timeline(origin);

    assert!(ultimate.arm(timeline.ultimate_deadline(), 17));
    assert!(!ultimate.arm(timeline.ultimate_deadline() + Duration::from_secs(1), 99));
    wait_for_count(&calls, 1);
    std::thread::sleep(Duration::from_millis(10));
    assert_eq!(calls.load(Ordering::Acquire), 1);
    assert_eq!(observed_code.load(Ordering::Acquire), 17);
    ultimate.stop_for_test();
}

#[test]
fn dropping_an_unarmed_or_disarmed_thread_is_clean() {
    let calls = Arc::new(AtomicUsize::new(0));
    let action_calls = Arc::clone(&calls);
    let origin = Instant::now();
    let mut ultimate = UltimateExit::initialize_for_test(
        origin,
        RawExitActions::testing(
            move |_| {
                action_calls.fetch_add(1, Ordering::AcqRel);
            },
            |_| {},
        ),
    )
    .expect("ultimate thread");
    assert!(ultimate.arm(origin + Duration::from_millis(100), 1));
    ultimate.stop_for_test();
    std::thread::sleep(Duration::from_millis(120));
    assert_eq!(calls.load(Ordering::Acquire), 0);
}

#[test]
fn panic_in_primary_action_invokes_the_fallback() {
    let fallbacks = Arc::new(AtomicUsize::new(0));
    let fallback_calls = Arc::clone(&fallbacks);
    let origin = Instant::now();
    let mut ultimate = UltimateExit::initialize_for_test(
        origin,
        RawExitActions::testing(
            |_| panic!("injected raw exit panic"),
            move |_| {
                fallback_calls.fetch_add(1, Ordering::AcqRel);
            },
        ),
    )
    .expect("ultimate thread");

    assert!(ultimate.arm(origin + Duration::from_millis(10), 3));
    wait_for_count(&fallbacks, 1);
    ultimate.stop_for_test();
}

#[test]
fn blocked_cef_shutdown_cannot_delay_the_ultimate_exit() {
    let origin = Instant::now();
    let calls = Arc::new(AtomicUsize::new(0));
    let action_calls = Arc::clone(&calls);
    let mut ultimate = UltimateExit::initialize_for_test(
        origin,
        RawExitActions::testing(
            move |_| {
                action_calls.fetch_add(1, Ordering::AcqRel);
            },
            |_| {},
        ),
    )
    .expect("ultimate thread");
    assert!(ultimate.arm(origin + Duration::from_millis(30), 1));

    let release = Arc::new((Mutex::new(false), Condvar::new()));
    let worker_release = Arc::clone(&release);
    let lifecycle = std::thread::spawn(move || {
        crate::startup::run_before_browser_shutdown(
            || 0,
            || {
                let (lock, wake) = &*worker_release;
                let mut released = lock.lock().expect("CEF test lock");
                while !*released {
                    released = wake.wait(released).expect("CEF test wait");
                }
            },
            || {},
        )
    });

    wait_for_count(&calls, 1);
    let (lock, wake) = &*release;
    *lock.lock().expect("CEF release lock") = true;
    wake.notify_all();
    assert_eq!(lifecycle.join().expect("CEF lifecycle"), 0);
    ultimate.stop_for_test();
}
