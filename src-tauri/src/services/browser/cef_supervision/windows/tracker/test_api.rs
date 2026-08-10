use super::WindowsCefTracker;
use std::time::{Duration, Instant};

impl WindowsCefTracker {
    pub(in crate::services::browser) fn close_gate_for_test(&self) -> bool {
        self.shared
            .emergency_close(Instant::now() + Duration::from_millis(50))
    }

    pub(in crate::services::browser) fn force_for_test(&self) {
        self.shared.emergency_force();
    }
}
