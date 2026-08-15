use super::error::OllamaErrorCode;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::Notify;
use tokio_util::sync::CancellationToken;

pub(crate) const OLLAMA_RECOVERY_RETRY_DELAYS: [Duration; 4] = [
    Duration::from_secs(5),
    Duration::from_secs(15),
    Duration::from_secs(60),
    Duration::from_secs(300),
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum RetryCategory {
    Recovery,
    #[cfg(test)]
    Validation,
    #[cfg(test)]
    Storage,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum RetryWait {
    Due,
    Cancelled,
    Closing,
}

#[derive(Clone)]
pub(crate) struct OllamaRecoveryRetry {
    inner: Arc<Mutex<RetryState>>,
}

struct RetryState {
    attempt: usize,
    timer_active: bool,
    closing: bool,
    wake: Arc<Notify>,
    last_log: Option<(OllamaErrorCode, RetryCategory)>,
}

impl OllamaRecoveryRetry {
    pub(crate) fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(RetryState {
                attempt: 0,
                timer_active: false,
                closing: false,
                wake: Arc::new(Notify::new()),
                last_log: None,
            })),
        }
    }

    pub(crate) fn reset_after_progress(&self) {
        self.lock().attempt = 0;
    }

    pub(crate) fn begin_timer(&self) -> Option<Duration> {
        let mut state = self.lock();
        if state.closing || state.timer_active {
            return None;
        }
        state.timer_active = true;
        let index = state.attempt.min(OLLAMA_RECOVERY_RETRY_DELAYS.len() - 1);
        Some(OLLAMA_RECOVERY_RETRY_DELAYS[index])
    }

    pub(crate) fn finish_timer(&self) {
        let mut state = self.lock();
        state.timer_active = false;
        state.attempt = state.attempt.saturating_add(1);
    }

    pub(crate) fn request_wake(&self) -> Result<(), OllamaErrorCode> {
        let wake = {
            let state = self.lock();
            if state.closing {
                return Err(OllamaErrorCode::OllamaClosing);
            }
            Arc::clone(&state.wake)
        };
        wake.notify_one();
        Ok(())
    }

    pub(crate) fn close(&self) {
        let wake = {
            let mut state = self.lock();
            state.closing = true;
            Arc::clone(&state.wake)
        };
        wake.notify_waiters();
    }

    pub(crate) fn is_closing(&self) -> bool {
        self.lock().closing
    }

    pub(crate) fn should_log(&self, code: OllamaErrorCode, category: RetryCategory) -> bool {
        let mut state = self.lock();
        let key = (code, category);
        if state.last_log == Some(key) {
            false
        } else {
            state.last_log = Some(key);
            true
        }
    }

    pub(crate) async fn wait(&self, cancellation: &CancellationToken) -> RetryWait {
        let Some(delay) = self.begin_timer() else {
            return if self.is_closing() {
                RetryWait::Closing
            } else {
                RetryWait::Due
            };
        };
        let wake = Arc::clone(&self.lock().wake);
        let outcome = tokio::select! {
            _ = cancellation.cancelled() => RetryWait::Cancelled,
            _ = wake.notified() => RetryWait::Due,
            _ = tokio::time::sleep(delay) => RetryWait::Due,
        };
        self.finish_timer();
        outcome
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, RetryState> {
        self.inner
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}
