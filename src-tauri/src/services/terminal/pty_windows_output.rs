use super::SharedChild;
use crate::services::terminal::exit_wait::{reap_child, ExitPoll};
use crate::services::terminal::normalize_exit_code;
use std::fs::File;
use std::io::{self, Read};
use std::os::windows::io::AsRawHandle;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use windows_sys::Win32::System::Pipes::PeekNamedPipe;

const OUTPUT_POLL_INTERVAL: Duration = Duration::from_millis(10);
const EXIT_DRAIN_GRACE: Duration = Duration::from_millis(100);

pub(super) struct OutputControl {
    file: Mutex<Option<File>>,
}

impl OutputControl {
    pub(super) fn new(file: File) -> Self {
        Self {
            file: Mutex::new(Some(file)),
        }
    }

    pub(super) fn close(&self) -> Result<(), String> {
        let file = self
            .file
            .lock()
            .map_err(|_| "terminal-error".to_string())?
            .take();
        drop(file);
        Ok(())
    }

    fn read_available(&self, buffer: &mut [u8]) -> io::Result<Option<usize>> {
        let file = self
            .file
            .lock()
            .map_err(|_| io::Error::other("terminal output lock"))?;
        let Some(file) = file.as_ref() else {
            return Ok(Some(0));
        };
        let mut available = 0_u32;
        // SAFETY: the File stays live under the mutex, and every optional
        // output pointer except `available` is intentionally null.
        let peeked = unsafe {
            PeekNamedPipe(
                file.as_raw_handle(),
                std::ptr::null_mut(),
                0,
                std::ptr::null_mut(),
                &mut available,
                std::ptr::null_mut(),
            )
        };
        if peeked == 0 {
            return Err(io::Error::last_os_error());
        }
        if available == 0 {
            return Ok(None);
        }
        let read_len = buffer.len().min(available as usize);
        let mut file_ref = file;
        file_ref.read(&mut buffer[..read_len]).map(Some)
    }
}

pub(super) struct PtyReader {
    output: Arc<OutputControl>,
    child: SharedChild,
    exited_at: Option<Instant>,
}

impl PtyReader {
    pub(super) fn new(output: Arc<OutputControl>, child: SharedChild) -> Self {
        Self {
            output,
            child,
            exited_at: None,
        }
    }

    fn child_finished(&self) -> bool {
        let Ok(mut child) = self.child.try_lock() else {
            return false;
        };
        child.try_wait().map_or(true, |status| status.is_some())
    }
}

impl Read for PtyReader {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        if buffer.is_empty() {
            return Ok(0);
        }
        loop {
            match self.output.read_available(buffer)? {
                Some(read) => {
                    if read > 0 && self.exited_at.is_some() {
                        self.exited_at = Some(Instant::now());
                    }
                    return Ok(read);
                }
                None if self.child_finished() => {
                    let exited_at = self.exited_at.get_or_insert_with(Instant::now);
                    if exited_at.elapsed() >= EXIT_DRAIN_GRACE {
                        self.output
                            .close()
                            .map_err(|_| io::Error::other("terminal output close"))?;
                        return Ok(0);
                    }
                }
                None => self.exited_at = None,
            }
            std::thread::sleep(OUTPUT_POLL_INTERVAL);
        }
    }
}

impl Drop for PtyReader {
    fn drop(&mut self) {
        let mut tree_killed = false;
        let mut root_killed = false;
        let _ = reap_child(|| match self.child.try_lock() {
            Ok(mut child) => match child.try_wait() {
                Ok(Some(status)) => ExitPoll::Exited(normalize_exit_code(status.code())),
                Ok(None) => {
                    if !tree_killed {
                        // Un lecteur illisible ne doit pas laisser son shell vivant.
                        crate::services::process_tree::kill(
                            child.id(),
                            crate::services::process_tree::ProcessKind::Terminal,
                        );
                        tree_killed = true;
                    }
                    if !root_killed {
                        let _ = child.kill();
                        root_killed = true;
                    }
                    ExitPoll::Running
                }
                Err(_) => {
                    if !tree_killed {
                        crate::services::process_tree::kill(
                            child.id(),
                            crate::services::process_tree::ProcessKind::Terminal,
                        );
                        tree_killed = true;
                    }
                    if !root_killed {
                        let _ = child.kill();
                        root_killed = true;
                    }
                    ExitPoll::Failed
                }
            },
            Err(std::sync::TryLockError::WouldBlock) => ExitPoll::Running,
            Err(std::sync::TryLockError::Poisoned(_)) => ExitPoll::Failed,
        });
        let _ = self.output.close();
    }
}
