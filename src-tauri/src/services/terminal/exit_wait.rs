use std::time::{Duration, Instant};

// Une seule horloge borne les lectures de statut et la recolte sur chaque OS.
const EXIT_STATUS_TIMEOUT: Duration = Duration::from_millis(200);
const EXIT_STATUS_POLL: Duration = Duration::from_millis(10);
const CHILD_REAP_TIMEOUT: Duration = Duration::from_secs(2);
const CHILD_REAP_POLL: Duration = Duration::from_millis(20);

pub(super) enum ExitPoll {
    Running,
    Exited(Option<u32>),
    Failed,
}

pub(super) fn wait_for_exit_code(mut poll: impl FnMut() -> ExitPoll) -> Option<u32> {
    poll_within(&mut poll, EXIT_STATUS_TIMEOUT, EXIT_STATUS_POLL)
}

pub(super) fn reap_child(mut poll: impl FnMut() -> ExitPoll) -> Option<u32> {
    poll_within(&mut poll, CHILD_REAP_TIMEOUT, CHILD_REAP_POLL)
}

fn poll_within(
    poll: &mut impl FnMut() -> ExitPoll,
    timeout: Duration,
    interval: Duration,
) -> Option<u32> {
    let deadline = Instant::now() + timeout;
    loop {
        match poll() {
            ExitPoll::Exited(code) => return code,
            ExitPoll::Failed => return None,
            ExitPoll::Running => {}
        }
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            return None;
        }
        std::thread::sleep(interval.min(remaining));
    }
}
