use super::manager::PtyChannelEvent;
use super::pty_session::PtySession;
use crate::services::work_registry::ServiceWorkAdmission;
use std::io::Read;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::JoinHandle;

pub(super) struct OwnedSession {
    session: PtySession,
    token: zeroize::Zeroizing<String>,
    reader_cancelled: Arc<AtomicBool>,
    reader: JoinHandle<()>,
    _admission: ServiceWorkAdmission<16>,
}

impl OwnedSession {
    pub(super) fn spawn(
        admission: ServiceWorkAdmission<16>,
        token: zeroize::Zeroizing<String>,
        cwd: Option<&str>,
        cols: u16,
        rows: u16,
        sink: impl Fn(PtyChannelEvent) + Send + 'static,
    ) -> Result<Self, String> {
        let (session, mut output) = PtySession::spawn(cwd, cols, rows)?;
        let status = session.child_status();
        let reader_cancelled = Arc::new(AtomicBool::new(false));
        let cancelled = Arc::clone(&reader_cancelled);
        let reader = std::thread::Builder::new()
            .name("beaver-pty-reader".to_string())
            .spawn(move || {
                let mut buffer = [0_u8; 4096];
                loop {
                    match output.read(&mut buffer) {
                        Ok(0) | Err(_) => break,
                        Ok(read) if !cancelled.load(Ordering::Acquire) => {
                            sink(PtyChannelEvent {
                                data: String::from_utf8_lossy(&buffer[..read]).to_string(),
                                is_exit: false,
                                exit_code: 0,
                            });
                        }
                        Ok(_) => {}
                    }
                }
                if !cancelled.load(Ordering::Acquire) {
                    sink(PtyChannelEvent {
                        data: String::new(),
                        is_exit: true,
                        exit_code: status.exit_code().unwrap_or(0),
                    });
                }
            })
            .map_err(|_| "terminal-error".to_string())?;
        Ok(Self {
            session,
            token,
            reader_cancelled,
            reader,
            _admission: admission,
        })
    }

    pub(super) fn token(&self) -> &str {
        &self.token
    }

    pub(super) fn write(&self, data: &[u8]) -> Result<(), String> {
        self.session.write(data)
    }

    pub(super) fn resize(&self, cols: u16, rows: u16) -> Result<(), String> {
        self.session.resize(cols, rows)
    }

    #[cfg(test)]
    pub(super) fn process_id(&self) -> Option<u32> {
        self.session.process_id()
    }

    pub(super) fn reader_finished(&self) -> bool {
        self.reader.is_finished()
    }

    pub(super) fn close(mut self) {
        self.reader_cancelled.store(true, Ordering::Release);
        let _ = self.session.shutdown();
        drop(self.session);
        if self.reader.join().is_err() {
            ::log::warn!("[terminal] lecteur PTY interrompu");
        }
    }
}
