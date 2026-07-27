use std::collections::VecDeque;
use std::sync::Mutex;
use std::time::{Duration, Instant};

const AUTO_RESTART_LIMIT: usize = 3;
const AUTO_RESTART_WINDOW: Duration = Duration::from_secs(300);

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
        while attempts
            .front()
            .is_some_and(|attempt| now.duration_since(*attempt) >= AUTO_RESTART_WINDOW)
        {
            attempts.pop_front();
        }
        if attempts.len() >= AUTO_RESTART_LIMIT {
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
