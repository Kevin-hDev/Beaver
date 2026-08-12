use std::sync::Mutex;
use std::time::{Duration, Instant};

const START_FAILURE_COOLDOWN: Duration = Duration::from_secs(30);
static LAST_START_FAILURE: Mutex<Option<StartFailure>> = Mutex::new(None);

struct StartFailure {
    at: Instant,
    message: String,
}

pub(super) fn recent() -> Option<String> {
    let guard = LAST_START_FAILURE.lock().ok()?;
    let failure = guard.as_ref()?;
    (failure.at.elapsed() < START_FAILURE_COOLDOWN).then(|| failure.message.clone())
}

pub(super) fn remember(error: &str) {
    if let Ok(mut guard) = LAST_START_FAILURE.lock() {
        *guard = Some(StartFailure {
            at: Instant::now(),
            message: error.to_string(),
        });
    }
}

pub(super) fn clear() {
    if let Ok(mut guard) = LAST_START_FAILURE.lock() {
        *guard = None;
    }
}

pub(super) fn safe_log_error(error: &str) -> String {
    error
        .chars()
        .map(|ch| if ch.is_control() { ' ' } else { ch })
        .take(240)
        .collect::<String>()
        .trim()
        .to_string()
}
