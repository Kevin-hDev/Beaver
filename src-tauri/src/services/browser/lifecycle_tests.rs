use super::lifecycle::{Lifecycle, RuntimePhase};

#[test]
fn engine_cannot_start_before_native_application_is_ready() {
    let mut lifecycle = Lifecycle::default();

    assert_eq!(lifecycle.phase(), RuntimePhase::Cold);
    assert!(!lifecycle.mark_running());
    assert_eq!(lifecycle.phase(), RuntimePhase::Failed);
}

#[test]
fn valid_lifecycle_starts_and_stops_once() {
    let mut lifecycle = Lifecycle::default();

    assert!(lifecycle.mark_application_prepared());
    assert!(!lifecycle.mark_application_prepared());
    assert!(lifecycle.mark_running());
    assert!(lifecycle.begin_stopping());
    assert!(!lifecycle.begin_stopping());
    assert!(lifecycle.mark_stopped());
    assert_eq!(lifecycle.phase(), RuntimePhase::Stopped);
}

#[test]
fn failed_security_gate_can_still_shutdown_cef() {
    let mut lifecycle = Lifecycle::default();

    assert!(lifecycle.mark_application_prepared());
    assert!(lifecycle.mark_failed());
    assert_eq!(lifecycle.phase(), RuntimePhase::Failed);
    assert!(lifecycle.begin_stopping());
    assert!(lifecycle.mark_stopped());
}

#[test]
fn cef_can_stop_while_security_gate_is_pending() {
    let mut lifecycle = Lifecycle::default();

    assert!(lifecycle.mark_application_prepared());
    assert!(lifecycle.begin_stopping());
    assert!(lifecycle.mark_stopped());
}
