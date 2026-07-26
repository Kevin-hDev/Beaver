use super::ffi_guard;
use std::sync::atomic::{AtomicBool, Ordering};

#[test]
fn returns_the_callback_value_when_no_panic_occurs() {
    assert_eq!(ffi_guard::value(7, || 42), 42);
}

#[test]
fn returns_the_fail_closed_value_when_a_callback_panics() {
    assert_eq!(ffi_guard::value(7, || panic!("test panic")), 7);
}

#[test]
fn executes_cleanup_after_a_void_callback_panic() {
    let failed = AtomicBool::new(false);
    ffi_guard::unit_or(
        || failed.store(true, Ordering::Release),
        || panic!("test panic"),
    );
    assert!(failed.load(Ordering::Acquire));
}
