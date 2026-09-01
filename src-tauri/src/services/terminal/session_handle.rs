use super::caller::TerminalOwner;
use super::manager::PtyManager;
use super::output_window::OutputWindow;
use super::public_error::not_authorized;
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

#[cfg(test)]
pub(super) struct NoopEmergencyStop;

#[cfg(test)]
impl EmergencyStop for NoopEmergencyStop {
    fn stop(&self) {}
}

pub(super) struct SessionControl {
    pub(super) output_window: Arc<OutputWindow>,
    pub(super) reader_cancelled: Arc<AtomicBool>,
    pub(super) reader_finished: Arc<AtomicBool>,
    pub(super) emergency_stop: Arc<dyn EmergencyStop>,
}

pub(super) struct ReaderFinishedGuard(pub(super) Arc<AtomicBool>);

impl Drop for ReaderFinishedGuard {
    fn drop(&mut self) {
        // Release publishes reader cleanup before SessionHandle observes the
        // completion with Acquire and removes this session from the manager.
        self.0.store(true, Ordering::Release);
    }
}

pub(super) struct SessionHandle {
    owner: TerminalOwner,
    token: zeroize::Zeroizing<String>,
    operations: Mutex<Option<Box<dyn SessionOps>>>,
    control: SessionControl,
    closing: AtomicBool,
}

impl SessionHandle {
    pub(super) fn new(
        owner: TerminalOwner,
        operations: Box<dyn SessionOps>,
        control: SessionControl,
        token: zeroize::Zeroizing<String>,
    ) -> Self {
        Self {
            owner,
            token,
            operations: Mutex::new(Some(operations)),
            control,
            closing: AtomicBool::new(false),
        }
    }

    pub(super) fn verify_owner(&self, owner: &TerminalOwner) -> Result<(), String> {
        (&self.owner == owner)
            .then_some(())
            .ok_or_else(not_authorized)
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

impl PtyManager {
    pub fn write(
        &self,
        owner: &TerminalOwner,
        id: u32,
        token: &str,
        data: &[u8],
    ) -> Result<(), String> {
        self.session(owner, id, token)?
            .with_live(|session| session.write(data))
    }

    pub fn resize(
        &self,
        owner: &TerminalOwner,
        id: u32,
        token: &str,
        cols: u16,
        rows: u16,
    ) -> Result<(), String> {
        self.session(owner, id, token)?
            .with_live(|session| session.resize(cols, rows))
    }

    pub fn acknowledge(
        &self,
        owner: &TerminalOwner,
        id: u32,
        token: &str,
        sequence: u32,
    ) -> Result<(), String> {
        self.session(owner, id, token)?.acknowledge(sequence)
    }

    pub fn kill(&self, owner: &TerminalOwner, id: u32, token: &str) -> Result<(), String> {
        let handle = self.session(owner, id, token)?;
        let removed = self
            .lock_state()?
            .sessions
            .remove(&id)
            .ok_or_else(not_found)?;
        debug_assert!(Arc::ptr_eq(&handle, &removed));
        removed.close();
        Ok(())
    }

    fn session(
        &self,
        owner: &TerminalOwner,
        id: u32,
        token: &str,
    ) -> Result<Arc<SessionHandle>, String> {
        let handle = self
            .lock_state()?
            .sessions
            .get(&id)
            .cloned()
            .ok_or_else(not_found)?;
        handle.verify_owner(owner)?;
        handle.verify_token(token)?;
        Ok(handle)
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
