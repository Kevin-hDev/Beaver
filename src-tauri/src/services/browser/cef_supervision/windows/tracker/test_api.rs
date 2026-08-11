use super::WindowsCefTracker;
use std::time::{Duration, Instant};

impl WindowsCefTracker {
    pub(in crate::services::browser) fn close_gate_for_test(&self) -> bool {
        let now = Instant::now();
        self.shared.emergency_close(
            now + Duration::from_millis(50),
            now + Duration::from_secs(2),
        )
    }

    pub(in crate::services::browser) fn force_for_test(&self) {
        self.shared.emergency_force();
    }

    pub(in crate::services::browser) fn expire_pending_with_probe_for_test(
        &self,
        slot: usize,
        resources_released: impl FnOnce(),
    ) -> bool {
        self.shared
            .pending
            .take(slot)
            .is_some_and(|pending| (*pending).expire_with_probe(resources_released))
    }
}
