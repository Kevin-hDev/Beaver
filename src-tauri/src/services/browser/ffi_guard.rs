use std::panic::{catch_unwind, AssertUnwindSafe};

pub(super) fn value<T>(fallback: T, operation: impl FnOnce() -> T) -> T {
    catch_unwind(AssertUnwindSafe(operation)).unwrap_or(fallback)
}

pub(super) fn unit(operation: impl FnOnce()) {
    let _ = catch_unwind(AssertUnwindSafe(operation));
}

pub(super) fn unit_or(on_panic: impl FnOnce(), operation: impl FnOnce()) {
    if catch_unwind(AssertUnwindSafe(operation)).is_err() {
        let _ = catch_unwind(AssertUnwindSafe(on_panic));
    }
}
