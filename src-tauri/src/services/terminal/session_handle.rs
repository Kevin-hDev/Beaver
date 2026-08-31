use super::output_window::OutputWindow;
use super::verify_token;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

pub(super) trait SessionOps: Send {
    fn write(&self, data: &[u8]) -> Result<(), String>;
    fn resize(&self, cols: u16, rows: u16) -> Result<(), String>;
    fn finish_close(self: Box<Self>);

    #[cfg(test)]
    fn process_id(&self) -> Option<u32> {
        None
    }
}

pub(super) trait EmergencyStop: Send + Sync {
    fn stop(&self);
}

pub(super) struct SessionControl {
    pub(super) output_window: Arc<OutputWindow>,
    pub(super) reader_cancelled: Arc<AtomicBool>,
    pub(super) reader_finished: Arc<AtomicBool>,
    pub(super) emergency_stop: Arc<dyn EmergencyStop>,
}

pub(super) struct SessionHandle {
    token: zeroize::Zeroizing<String>,
    operations: Mutex<Option<Box<dyn SessionOps>>>,
    control: SessionControl,
    closing: AtomicBool,
}

impl SessionHandle {
    pub(super) fn new(
        operations: Box<dyn SessionOps>,
        control: SessionControl,
        token: zeroize::Zeroizing<String>,
    ) -> Self {
        Self {
            token,
            operations: Mutex::new(Some(operations)),
            control,
            closing: AtomicBool::new(false),
        }
    }

    pub(super) fn verify_token(&self, token: &str) -> Result<(), String> {
        verify_token(&self.token, token)
    }

    pub(super) fn with_live<Output>(
        &self,
        operation: impl FnOnce(&dyn SessionOps) -> Result<Output, String>,
    ) -> Result<Output, String> {
        if self.closing.load(Ordering::Acquire) {
            return Err(not_found());
        }
        let operations = self.operations.lock().map_err(|_| terminal_error())?;
        if self.closing.load(Ordering::Acquire) {
            return Err(not_found());
        }
        operation(operations.as_deref().ok_or_else(not_found)?)
    }

    pub(super) fn reader_finished(&self) -> bool {
        // Acquire observes every reader action sequenced before the guard's
        // Release store, so reaping cannot race past final reader cleanup.
        self.control.reader_finished.load(Ordering::Acquire)
    }

    #[allow(
        dead_code,
        reason = "wired to the IPC acknowledgement command in Task 4"
    )]
    pub(super) fn acknowledge(&self, sequence: u32) -> Result<(), String> {
        self.control.output_window.acknowledge(sequence)
    }

    pub(super) fn close(&self) {
        if self
            .closing
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_err()
        {
            return;
        }
        self.control.reader_cancelled.store(true, Ordering::Release);
        self.control.output_window.close();
        self.control.emergency_stop.stop();
        let mut operations = self
            .operations
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let operations = operations.take();
        if let Some(operations) = operations {
            operations.finish_close();
        }
    }

    #[cfg(test)]
    pub(super) fn process_id(&self) -> Option<u32> {
        self.with_live(|operations| Ok(operations.process_id()))
            .ok()
            .flatten()
    }
}

impl Drop for SessionHandle {
    fn drop(&mut self) {
        self.close();
    }
}

fn not_found() -> String {
    "terminal-not-found".to_string()
}

fn terminal_error() -> String {
    "terminal-error".to_string()
}
