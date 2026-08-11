use super::emergency::{EmergencyInventory, VerifiedProcessIdentity};
use super::emergency_drain::{EmergencyObservation, EmergencySignaler};
use super::policy::{ShutdownPolicy, ShutdownTimeline};
use super::state::{ShutdownPhase, ShutdownState};
use super::ultimate::{RawExitActions, UltimateExit};
use super::watchdog::{WatchdogActions, WatchdogThread};
use super::ExitIntent;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, Instant};

fn timeline(origin: Instant) -> ShutdownTimeline {
    ShutdownTimeline::from_origin(
        origin,
        ShutdownPolicy::new(
            Duration::from_millis(5),
            Duration::from_millis(10),
            Duration::from_millis(20),
            Duration::from_millis(40),
        )
        .expect("watchdog policy"),
    )
}

fn wait_until(condition: impl Fn() -> bool) {
    let deadline = Instant::now() + Duration::from_secs(1);
    while !condition() {
        assert!(Instant::now() < deadline, "condition deadline");
        std::thread::yield_now();
    }
}

struct CountingSignaler {
    calls: AtomicUsize,
}

impl EmergencySignaler for CountingSignaler {
    fn signal_or_recheck(
        &self,
        _identity: VerifiedProcessIdentity,
        _already_requested: bool,
    ) -> EmergencyObservation {
        self.calls.fetch_add(1, Ordering::AcqRel);
        EmergencyObservation::Terminating
    }
}

#[test]
fn watchdog_requests_tauri_exit_then_starts_emergency_drain() {
    let origin = Instant::now();
    let timeline = timeline(origin);
    let state = Arc::new(ShutdownState::new());
    assert_eq!(state.begin_closing(), super::state::BeginClosing::Started);
    let inventory = EmergencyInventory::new();
    let _registration = inventory
        .try_publish(VerifiedProcessIdentity::new(5, 15, 25).expect("identity"))
        .expect("registration");
    let exits = Arc::new(AtomicUsize::new(0));
    let exit_calls = Arc::clone(&exits);
    let signaler = Arc::new(CountingSignaler {
        calls: AtomicUsize::new(0),
    });
    let actions = WatchdogActions::testing(
        move |intent, _| {
            assert_eq!(intent, ExitIntent::Restart);
            exit_calls.fetch_add(1, Ordering::AcqRel);
        },
        signaler.clone(),
    );
    let watchdog = WatchdogThread::spawn(
        timeline,
        Arc::clone(&state),
        inventory,
        ExitIntent::Restart,
        9,
        actions,
    )
    .expect("watchdog");

    wait_until(|| exits.load(Ordering::Acquire) == 1);
    assert_eq!(state.phase(), ShutdownPhase::ReadyToExit);
    wait_until(|| signaler.calls.load(Ordering::Acquire) > 0);
    watchdog.join_for_test();
}

#[test]
fn watchdog_spawn_failure_does_not_touch_existing_state() {
    let origin = Instant::now();
    let state = Arc::new(ShutdownState::new());
    assert_eq!(state.begin_closing(), super::state::BeginClosing::Started);
    let result = WatchdogThread::spawn_with(
        timeline(origin),
        Arc::clone(&state),
        EmergencyInventory::new(),
        ExitIntent::Exit,
        0,
        WatchdogActions::testing(
            |_, _| {},
            Arc::new(CountingSignaler {
                calls: AtomicUsize::new(0),
            }),
        ),
        |_| Err(std::io::Error::other("injected watchdog spawn failure")),
    );

    assert!(result.is_err());
    assert_eq!(state.phase(), ShutdownPhase::Closing);
}

struct BlockingSignaler {
    entered: AtomicBool,
    released: Mutex<bool>,
    wake: Condvar,
}

impl BlockingSignaler {
    fn release(&self) {
        let mut released = self.released.lock().expect("release lock");
        *released = true;
        self.wake.notify_all();
    }
}

impl EmergencySignaler for BlockingSignaler {
    fn signal_or_recheck(
        &self,
        _identity: VerifiedProcessIdentity,
        _already_requested: bool,
    ) -> EmergencyObservation {
        self.entered.store(true, Ordering::Release);
        let mut released = self.released.lock().expect("blocking lock");
        while !*released {
            released = self.wake.wait(released).expect("blocking wait");
        }
        EmergencyObservation::Exited
    }
}

#[test]
fn blocked_watchdog_cannot_delay_the_ultimate_exit() {
    let origin = Instant::now();
    let timeline = timeline(origin);
    let ultimate_calls = Arc::new(AtomicUsize::new(0));
    let raw_calls = Arc::clone(&ultimate_calls);
    let mut ultimate = UltimateExit::initialize_for_test(
        origin,
        RawExitActions::testing(
            move |_| {
                raw_calls.fetch_add(1, Ordering::AcqRel);
            },
            |_| {},
        ),
    )
    .expect("ultimate");
    assert!(ultimate.arm(timeline.ultimate_deadline(), 1));

    let state = Arc::new(ShutdownState::new());
    assert_eq!(state.begin_closing(), super::state::BeginClosing::Started);
    let inventory = EmergencyInventory::new();
    let _registration = inventory
        .try_publish(VerifiedProcessIdentity::new(8, 18, 28).expect("identity"))
        .expect("registration");
    let signaler = Arc::new(BlockingSignaler {
        entered: AtomicBool::new(false),
        released: Mutex::new(false),
        wake: Condvar::new(),
    });
    let actions = WatchdogActions::testing(|_, _| {}, signaler.clone());
    let watchdog = WatchdogThread::spawn(timeline, state, inventory, ExitIntent::Exit, 0, actions)
        .expect("watchdog");

    wait_until(|| signaler.entered.load(Ordering::Acquire));
    wait_until(|| ultimate_calls.load(Ordering::Acquire) == 1);
    signaler.release();
    watchdog.join_for_test();
    ultimate.stop_for_test();
}
