use std::sync::{Condvar, Mutex};

pub(super) struct OutputWindow {
    closed: Mutex<bool>,
    changed: Condvar,
}

impl OutputWindow {
    pub(super) fn new() -> Self {
        Self {
            closed: Mutex::new(false),
            changed: Condvar::new(),
        }
    }

    pub(super) fn acknowledge(&self, _sequence: u32) -> Result<(), String> {
        if *self.closed.lock().map_err(|_| terminal_error())? {
            Err(not_found())
        } else {
            Ok(())
        }
    }

    pub(super) fn close(&self) {
        let mut closed = self
            .closed
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        *closed = true;
        self.changed.notify_all();
    }

    #[cfg(test)]
    pub(super) fn wait_until_closed_for_test(&self) -> Result<(), String> {
        let mut closed = self.closed.lock().map_err(|_| terminal_error())?;
        while !*closed {
            closed = self.changed.wait(closed).map_err(|_| terminal_error())?;
        }
        Err(not_found())
    }
}

impl Default for OutputWindow {
    fn default() -> Self {
        Self::new()
    }
}

fn not_found() -> String {
    "terminal-not-found".to_string()
}

fn terminal_error() -> String {
    "terminal-error".to_string()
}
