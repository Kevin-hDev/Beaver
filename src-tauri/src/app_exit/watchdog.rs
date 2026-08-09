use super::emergency::{EmergencyInventory, VerifiedProcessIdentity};
use super::emergency_drain::{EmergencyObservation, EmergencySignaler};
use super::policy::{post_loop_sweep_timeout, watchdog_recheck_interval, ShutdownTimeline};
use super::state::ShutdownState;
use std::io;
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::Instant;

type ExitRequest = Arc<dyn Fn(i32) + Send + Sync + 'static>;

struct UnadoptedSignaler;

impl EmergencySignaler for UnadoptedSignaler {
    fn signal_or_recheck(
        &self,
        _identity: VerifiedProcessIdentity,
        _already_requested: bool,
    ) -> EmergencyObservation {
        EmergencyObservation::IdentityMismatch
    }
}

pub(super) struct WatchdogActions {
    request_exit: ExitRequest,
    signaler: Arc<dyn EmergencySignaler>,
}

impl WatchdogActions {
    pub(super) fn production(request_exit: impl Fn(i32) + Send + Sync + 'static) -> Self {
        Self {
            request_exit: Arc::new(request_exit),
            signaler: Arc::new(UnadoptedSignaler),
        }
    }

    #[cfg(test)]
    pub(super) fn testing(
        request_exit: impl Fn(i32) + Send + Sync + 'static,
        signaler: Arc<dyn EmergencySignaler>,
    ) -> Self {
        Self {
            request_exit: Arc::new(request_exit),
            signaler,
        }
    }
}

pub(super) struct WatchdogThread {
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "production deliberately detaches the watchdog thread"
        )
    )]
    join: Option<JoinHandle<()>>,
}

impl WatchdogThread {
    pub(super) fn spawn(
        timeline: ShutdownTimeline,
        state: Arc<ShutdownState>,
        inventory: EmergencyInventory,
        exit_code: i32,
        actions: WatchdogActions,
    ) -> io::Result<Self> {
        Self::spawn_inner(timeline, state, inventory, exit_code, actions, |worker| {
            std::thread::Builder::new()
                .name("beaver-shutdown-watchdog".to_string())
                .spawn(move || worker())
        })
    }

    #[cfg(test)]
    pub(super) fn spawn_with<Spawn>(
        timeline: ShutdownTimeline,
        state: Arc<ShutdownState>,
        inventory: EmergencyInventory,
        exit_code: i32,
        actions: WatchdogActions,
        spawn: Spawn,
    ) -> io::Result<Self>
    where
        Spawn: FnOnce(Box<dyn FnOnce() + Send + 'static>) -> io::Result<JoinHandle<()>>,
    {
        Self::spawn_inner(timeline, state, inventory, exit_code, actions, spawn)
    }

    fn spawn_inner<Spawn>(
        timeline: ShutdownTimeline,
        state: Arc<ShutdownState>,
        inventory: EmergencyInventory,
        exit_code: i32,
        actions: WatchdogActions,
        spawn: Spawn,
    ) -> io::Result<Self>
    where
        Spawn: FnOnce(Box<dyn FnOnce() + Send + 'static>) -> io::Result<JoinHandle<()>>,
    {
        let worker =
            Box::new(move || run_watchdog(timeline, &state, &inventory, exit_code, &actions));
        Ok(Self {
            join: Some(spawn(worker)?),
        })
    }

    #[cfg(test)]
    pub(super) fn join_for_test(mut self) {
        if let Some(join) = self.join.take() {
            join.join().expect("watchdog thread");
        }
    }
}

fn run_watchdog(
    timeline: ShutdownTimeline,
    state: &ShutdownState,
    inventory: &EmergencyInventory,
    exit_code: i32,
    actions: &WatchdogActions,
) {
    wait_until(timeline.tauri_exit_deadline());
    if state.mark_ready() {
        (actions.request_exit)(exit_code);
    }
    wait_until(timeline.emergency_deadline());
    while Instant::now() < timeline.ultimate_deadline() {
        inventory.drain_once(actions.signaler.as_ref());
        let remaining = timeline.remaining_until(timeline.ultimate_deadline());
        if remaining.is_zero() {
            break;
        }
        std::thread::park_timeout(remaining.min(watchdog_recheck_interval()));
    }
}

fn wait_until(deadline: Instant) {
    loop {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            return;
        }
        std::thread::park_timeout(remaining);
    }
}

pub(super) fn drain_post_loop(inventory: &EmergencyInventory, timeline: ShutdownTimeline) {
    let signaler = UnadoptedSignaler;
    let local_limit = Instant::now() + post_loop_sweep_timeout();
    let deadline = local_limit.min(timeline.emergency_deadline());
    while inventory.has_active() && Instant::now() < deadline {
        inventory.drain_once(&signaler);
        let remaining = deadline.saturating_duration_since(Instant::now());
        if !remaining.is_zero() && inventory.has_active() {
            std::thread::park_timeout(remaining.min(watchdog_recheck_interval()));
        }
    }
}
