use std::time::Duration;

use super::wait_for_parent;

#[test]
fn accepts_a_parent_that_has_already_exited() {
    assert!(wait_for_parent(u32::MAX, Duration::from_millis(10)).is_ok());
}

#[test]
fn refuses_invalid_or_still_running_parent() {
    assert!(wait_for_parent(0, Duration::from_millis(10)).is_err());
    assert!(wait_for_parent(std::process::id(), Duration::from_millis(10)).is_err());
}
