use super::types::{HOST_RESTART_WINDOW_SECONDS, MAX_HOST_RESTARTS_PER_WINDOW};
use std::collections::VecDeque;
use std::sync::Mutex;
use std::time::{Duration, Instant};

#[derive(Default)]
pub struct RestartBudget {
    attempts: Mutex<VecDeque<Instant>>,
}

impl RestartBudget {
    pub fn allow(&self) -> bool {
        let Ok(mut attempts) = self.attempts.lock() else {
            return false;
        };
        let now = Instant::now();
        while attempts.front().is_some_and(|attempt| {
            now.duration_since(*attempt) >= Duration::from_secs(HOST_RESTART_WINDOW_SECONDS as u64)
        }) {
            attempts.pop_front();
        }
        if attempts.len() >= MAX_HOST_RESTARTS_PER_WINDOW {
            return false;
        }
        attempts.push_back(now);
        true
    }

    pub fn reset(&self) {
        if let Ok(mut attempts) = self.attempts.lock() {
            attempts.clear();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::RestartBudget;

    #[test]
    fn bounds_automatic_restarts_until_manual_reset() {
        let budget = RestartBudget::default();

        assert!(budget.allow());
        assert!(budget.allow());
        assert!(budget.allow());
        assert!(!budget.allow());
        budget.reset();
        assert!(budget.allow());
    }
}
