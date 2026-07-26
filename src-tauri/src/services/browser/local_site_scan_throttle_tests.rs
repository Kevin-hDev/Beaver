use super::local_site_scan_throttle::{LocalSiteScanThrottle, MIN_SCAN_INTERVAL};
use std::time::{Duration, Instant};

#[test]
fn scan_throttle_allows_the_first_scan_and_enforces_the_backend_interval() {
    let start = Instant::now();
    let mut throttle = LocalSiteScanThrottle::default();

    assert!(throttle.allow(start));
    assert!(!throttle.allow(start + MIN_SCAN_INTERVAL - Duration::from_millis(1)));
    assert!(throttle.allow(start + MIN_SCAN_INTERVAL));
}
