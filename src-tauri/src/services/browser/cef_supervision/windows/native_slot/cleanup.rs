use super::super::super::CefUnavailableCategory;
use super::super::native_state::{
    close, NATIVE_ADMITTED, NATIVE_CLEANING, NATIVE_EXITED, NATIVE_FREE, NATIVE_INSPECTING,
    NATIVE_PREPARED, NATIVE_TERMINATING,
};
use super::WindowsNativeSlot;
use std::sync::atomic::Ordering;

impl WindowsNativeSlot {
    pub(super) fn claim_inspection(&self) -> Result<u8, CefUnavailableCategory> {
        loop {
            let state = self.state.load(Ordering::Acquire);
            match state {
                NATIVE_EXITED | NATIVE_INSPECTING => return Ok(state),
                NATIVE_ADMITTED | NATIVE_TERMINATING => {
                    if self
                        .state
                        .compare_exchange(
                            state,
                            NATIVE_INSPECTING,
                            Ordering::AcqRel,
                            Ordering::Acquire,
                        )
                        .is_ok()
                    {
                        return Ok(state);
                    }
                }
                _ => return Err(CefUnavailableCategory::Admission),
            }
        }
    }

    pub(super) fn transition(
        &self,
        generation: u64,
        from: u8,
        to: u8,
    ) -> Result<(), CefUnavailableCategory> {
        if self.generation.load(Ordering::Acquire) != generation {
            return Err(CefUnavailableCategory::Admission);
        }
        self.state
            .compare_exchange(from, to, Ordering::AcqRel, Ordering::Acquire)
            .map(|_| ())
            .map_err(|_| CefUnavailableCategory::Admission)
    }

    pub(super) fn cleanup_prepared(&self, generation: u64) {
        if self
            .transition(generation, NATIVE_PREPARED, NATIVE_CLEANING)
            .is_ok()
        {
            self.cleanup();
        }
    }

    pub(super) fn cleanup_exited(&self, generation: u64) {
        if self
            .transition(generation, NATIVE_EXITED, NATIVE_CLEANING)
            .is_ok()
        {
            self.cleanup();
        }
    }

    fn cleanup(&self) {
        close(self.job.swap(std::ptr::null_mut(), Ordering::AcqRel));
        close(self.process.swap(std::ptr::null_mut(), Ordering::AcqRel));
        self.pid.store(0, Ordering::Relaxed);
        self.parent_pid.store(0, Ordering::Relaxed);
        self.started_at.store(0, Ordering::Relaxed);
        self.generation.store(0, Ordering::Relaxed);
        self.state.store(NATIVE_FREE, Ordering::Release);
    }
}
