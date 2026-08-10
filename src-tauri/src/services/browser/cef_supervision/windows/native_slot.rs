use super::super::CefUnavailableCategory;
use super::confinement::WindowsConfinement;
use super::native_authority::{classify_termination, WindowsTerminationState};
use super::native_state::{
    close, state_for, NATIVE_ADMITTED, NATIVE_CLEANING, NATIVE_EXITED, NATIVE_FREE,
    NATIVE_INSPECTING, NATIVE_PREPARED, NATIVE_TERMINATING, NATIVE_WRITING,
};
use std::ffi::c_void;
use std::sync::atomic::{AtomicBool, AtomicPtr, AtomicU32, AtomicU64, AtomicU8, Ordering};
use windows_sys::Win32::Foundation::{WAIT_OBJECT_0, WAIT_TIMEOUT};
use windows_sys::Win32::System::Threading::{TerminateProcess, WaitForSingleObject};

pub(super) struct WindowsNativeSlot {
    state: AtomicU8,
    generation: AtomicU64,
    pid: AtomicU32,
    parent_pid: AtomicU32,
    started_at: AtomicU64,
    process: AtomicPtr<c_void>,
    job: AtomicPtr<c_void>,
    termination_sent: AtomicBool,
    release_requested: AtomicBool,
}

impl WindowsNativeSlot {
    pub(super) fn new() -> Self {
        Self {
            state: AtomicU8::new(NATIVE_FREE),
            generation: AtomicU64::new(0),
            pid: AtomicU32::new(0),
            parent_pid: AtomicU32::new(0),
            started_at: AtomicU64::new(0),
            process: AtomicPtr::new(std::ptr::null_mut()),
            job: AtomicPtr::new(std::ptr::null_mut()),
            termination_sent: AtomicBool::new(false),
            release_requested: AtomicBool::new(false),
        }
    }

    pub(super) fn prepare(
        &self,
        generation: u64,
        confinement: WindowsConfinement,
    ) -> Result<(), CefUnavailableCategory> {
        if generation == 0
            || self
                .state
                .compare_exchange(
                    NATIVE_FREE,
                    NATIVE_WRITING,
                    Ordering::AcqRel,
                    Ordering::Acquire,
                )
                .is_err()
        {
            return Err(CefUnavailableCategory::Admission);
        }
        let parts = confinement.into_raw();
        self.generation.store(generation, Ordering::Relaxed);
        self.pid.store(parts.process.pid, Ordering::Relaxed);
        self.parent_pid
            .store(parts.process.parent_pid, Ordering::Relaxed);
        self.started_at
            .store(parts.process.started_at, Ordering::Relaxed);
        self.process.store(parts.process.handle, Ordering::Relaxed);
        self.job.store(parts.job, Ordering::Relaxed);
        self.termination_sent.store(false, Ordering::Relaxed);
        self.release_requested.store(false, Ordering::Relaxed);
        self.state.store(NATIVE_PREPARED, Ordering::Release);
        Ok(())
    }

    pub(super) fn mark_admitted(&self, generation: u64) -> Result<(), CefUnavailableCategory> {
        self.transition(generation, NATIVE_PREPARED, NATIVE_ADMITTED)
    }

    pub(super) fn inspect(
        &self,
        generation: u64,
        request_shutdown: bool,
    ) -> Result<WindowsTerminationState, CefUnavailableCategory> {
        if self.generation.load(Ordering::Acquire) != generation {
            return Err(CefUnavailableCategory::Admission);
        }
        let previous = self.claim_inspection()?;
        if previous == NATIVE_EXITED {
            return Ok(WindowsTerminationState::Exited);
        }
        if previous == NATIVE_INSPECTING {
            return Ok(WindowsTerminationState::Terminating);
        }
        let process = self.process.load(Ordering::Acquire);
        if process.is_null() {
            self.state.store(previous, Ordering::Release);
            return Err(CefUnavailableCategory::Admission);
        }
        let shutdown = request_shutdown || previous == NATIVE_TERMINATING;
        if request_shutdown && !self.termination_sent.swap(true, Ordering::AcqRel) {
            close(self.job.swap(std::ptr::null_mut(), Ordering::AcqRel));
            let _ = unsafe { TerminateProcess(process, 1) };
        }
        let signaled = match unsafe { WaitForSingleObject(process, 0) } {
            WAIT_OBJECT_0 => true,
            WAIT_TIMEOUT => false,
            _ => {
                self.state.store(previous, Ordering::Release);
                return Err(CefUnavailableCategory::Admission);
            }
        };
        let result = classify_termination(shutdown, signaled);
        self.state.store(state_for(result), Ordering::Release);
        if self.release_requested.load(Ordering::Acquire) {
            if result == WindowsTerminationState::Exited {
                self.cleanup_exited(generation);
            } else if !shutdown {
                return self.inspect(generation, true);
            }
        }
        Ok(result)
    }

    pub(super) fn release(&self, generation: u64) {
        if self.generation.load(Ordering::Acquire) != generation {
            return;
        }
        self.release_requested.store(true, Ordering::Release);
        match self.state.load(Ordering::Acquire) {
            NATIVE_PREPARED => self.cleanup_prepared(generation),
            NATIVE_ADMITTED | NATIVE_TERMINATING => {
                let _ = self.inspect(generation, true);
            }
            NATIVE_EXITED => self.cleanup_exited(generation),
            NATIVE_INSPECTING | NATIVE_WRITING | NATIVE_CLEANING | NATIVE_FREE => {}
            _ => {}
        }
    }

    pub(super) fn refresh(&self) -> Result<(), CefUnavailableCategory> {
        let state = self.state.load(Ordering::Acquire);
        let generation = self.generation.load(Ordering::Acquire);
        match state {
            NATIVE_ADMITTED | NATIVE_TERMINATING => {
                let _ = self.inspect(generation, false)?;
            }
            NATIVE_EXITED if self.release_requested.load(Ordering::Acquire) => {
                self.cleanup_exited(generation);
            }
            _ => {}
        }
        Ok(())
    }

    pub(super) fn is_occupied(&self) -> bool {
        self.state.load(Ordering::Acquire) != NATIVE_FREE
    }

    pub(super) fn force_close(&self) {
        close(self.job.swap(std::ptr::null_mut(), Ordering::AcqRel));
        close(self.process.swap(std::ptr::null_mut(), Ordering::AcqRel));
        self.state.store(NATIVE_FREE, Ordering::Release);
    }

    fn claim_inspection(&self) -> Result<u8, CefUnavailableCategory> {
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

    fn transition(&self, generation: u64, from: u8, to: u8) -> Result<(), CefUnavailableCategory> {
        if self.generation.load(Ordering::Acquire) != generation {
            return Err(CefUnavailableCategory::Admission);
        }
        self.state
            .compare_exchange(from, to, Ordering::AcqRel, Ordering::Acquire)
            .map(|_| ())
            .map_err(|_| CefUnavailableCategory::Admission)
    }

    fn cleanup_prepared(&self, generation: u64) {
        if self
            .transition(generation, NATIVE_PREPARED, NATIVE_CLEANING)
            .is_ok()
        {
            self.cleanup();
        }
    }

    fn cleanup_exited(&self, generation: u64) {
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
