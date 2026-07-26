use std::time::{Duration, Instant};

pub(super) const MIN_SCAN_INTERVAL: Duration = Duration::from_secs(4);

#[derive(Default)]
pub(super) struct LocalSiteScanThrottle {
    last_scan: Option<Instant>,
}

impl LocalSiteScanThrottle {
    pub(super) fn allow(&mut self, now: Instant) -> bool {
        let allowed = self.last_scan.is_none_or(|last_scan| {
            now.checked_duration_since(last_scan)
                .is_some_and(|elapsed| elapsed >= MIN_SCAN_INTERVAL)
        });
        if allowed {
            self.last_scan = Some(now);
        }
        allowed
    }
}
