use std::sync::LazyLock;
use std::time::{Duration, Instant};

use tokio::sync::{Mutex, MutexGuard};

use super::types::{connection_index, CONNECTION_COUNT};

const MIN_FORCE_INTERVAL: Duration = Duration::from_secs(2);

#[derive(Default)]
pub(super) struct GateState {
    last_completed: Option<Instant>,
}

static GATES: LazyLock<[Mutex<GateState>; CONNECTION_COUNT]> =
    LazyLock::new(|| std::array::from_fn(|_| Mutex::new(GateState::default())));

pub async fn lock(connection_id: &str) -> Option<MutexGuard<'static, GateState>> {
    Some(GATES[connection_index(connection_id)?].lock().await)
}

pub fn should_skip(state: &GateState, requested_at: Instant, force_refresh: bool) -> bool {
    let Some(completed) = state.last_completed else {
        return false;
    };
    completed >= requested_at || (force_refresh && completed.elapsed() < MIN_FORCE_INTERVAL)
}

pub fn complete(state: &mut GateState) {
    state.last_completed = Some(Instant::now());
}

pub fn reset(state: &mut GateState) {
    state.last_completed = None;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collapses_joined_and_too_frequent_forced_requests() {
        let requested = Instant::now();
        let mut state = GateState::default();
        complete(&mut state);
        assert!(should_skip(&state, requested, false));
        assert!(should_skip(&state, Instant::now(), true));
        reset(&mut state);
        assert!(!should_skip(&state, Instant::now(), true));
    }
}
