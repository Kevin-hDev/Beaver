use std::sync::atomic::{AtomicUsize, Ordering};

static SANDBOX_INFO: AtomicUsize = AtomicUsize::new(0);

pub(crate) fn register(info: *mut u8) -> bool {
    if info.is_null() {
        return false;
    }
    SANDBOX_INFO
        .compare_exchange(0, info as usize, Ordering::AcqRel, Ordering::Acquire)
        .is_ok()
}

pub(super) fn get() -> Option<*mut u8> {
    let value = SANDBOX_INFO.load(Ordering::Acquire);
    (value != 0).then_some(value as *mut u8)
}
