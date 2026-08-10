use std::time::{Duration, Instant};

const GRACEFUL_TIMEOUT: Duration = Duration::from_secs(8);
const TAURI_EXIT_TIMEOUT: Duration = Duration::from_secs(10);
const POST_LOOP_SWEEP_TIMEOUT: Duration = Duration::from_secs(3);
const EMERGENCY_TIMEOUT: Duration = Duration::from_secs(13);
const ULTIMATE_EXIT_TIMEOUT: Duration = Duration::from_secs(15);
const CEF_ADMISSION_BARRIER_TIMEOUT: Duration = Duration::from_millis(50);
const CEF_HELPER_EXIT_MARGIN: Duration = Duration::from_secs(1);
const WATCHDOG_RECHECK_INTERVAL: Duration = Duration::from_millis(10);

pub(super) const fn watchdog_recheck_interval() -> Duration {
    WATCHDOG_RECHECK_INTERVAL
}

pub(super) const fn post_loop_sweep_timeout() -> Duration {
    POST_LOOP_SWEEP_TIMEOUT
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct ShutdownPolicy {
    graceful: Duration,
    tauri_exit: Duration,
    emergency: Duration,
    ultimate: Duration,
}

impl ShutdownPolicy {
    pub(super) fn production() -> Self {
        Self {
            graceful: GRACEFUL_TIMEOUT,
            tauri_exit: TAURI_EXIT_TIMEOUT,
            emergency: EMERGENCY_TIMEOUT,
            ultimate: ULTIMATE_EXIT_TIMEOUT,
        }
    }

    #[cfg(test)]
    pub(super) fn new(
        graceful: Duration,
        tauri_exit: Duration,
        emergency: Duration,
        ultimate: Duration,
    ) -> Option<Self> {
        (Duration::ZERO < graceful
            && graceful < tauri_exit
            && tauri_exit < emergency
            && emergency < ultimate)
            .then_some(Self {
                graceful,
                tauri_exit,
                emergency,
                ultimate,
            })
    }

    pub(super) fn graceful(self) -> Duration {
        self.graceful
    }

    pub(super) fn tauri_exit(self) -> Duration {
        self.tauri_exit
    }

    pub(super) fn emergency(self) -> Duration {
        self.emergency
    }

    pub(super) fn ultimate(self) -> Duration {
        self.ultimate
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct ShutdownTimeline {
    origin: Instant,
    policy: ShutdownPolicy,
}

impl ShutdownTimeline {
    pub(super) fn from_origin(origin: Instant, policy: ShutdownPolicy) -> Self {
        Self { origin, policy }
    }

    pub(super) fn graceful_deadline(self) -> Instant {
        self.origin + self.policy.graceful()
    }

    pub(super) fn cef_admission_deadline(self) -> Instant {
        self.origin + CEF_ADMISSION_BARRIER_TIMEOUT
    }

    pub(super) fn cef_helper_exit_deadline(self) -> Instant {
        let derived_margin = self.policy.ultimate() / 15;
        self.ultimate_deadline() - CEF_HELPER_EXIT_MARGIN.min(derived_margin)
    }

    pub(super) fn tauri_exit_deadline(self) -> Instant {
        self.origin + self.policy.tauri_exit()
    }

    pub(super) fn emergency_deadline(self) -> Instant {
        self.origin + self.policy.emergency()
    }

    pub(super) fn ultimate_deadline(self) -> Instant {
        self.origin + self.policy.ultimate()
    }

    pub(super) fn remaining_at(self, deadline: Instant, now: Instant) -> Duration {
        deadline.saturating_duration_since(now)
    }

    pub(super) fn remaining_until(self, deadline: Instant) -> Duration {
        self.remaining_at(deadline, Instant::now())
    }
}
