use super::output_window::{spawn_reader, OutputWindow};
use super::pty_session::PtySession;
use super::public_error::terminal_error;
use super::session_handle::{EmergencyStop, SessionControl, SessionOps};
use super::PtyChannelEvent;
use crate::services::work_registry::ServiceWorkAdmission;
use std::io::Read;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::JoinHandle;

pub(super) struct OwnedSession {
    session: PtySession,
    reader: JoinHandle<()>,
    _admission: ServiceWorkAdmission<16>,
}

struct ProcessEmergencyStop {
    pid: u32,
    stopped: AtomicBool,
}

impl EmergencyStop for ProcessEmergencyStop {
    fn stop(&self) {
        if self.stopped.swap(true, Ordering::AcqRel) {
            return;
        }
        crate::services::process_tree::kill(
            self.pid,
            crate::services::process_tree::ProcessKind::Terminal,
        );
    }
}

impl OwnedSession {
    pub(super) fn spawn(
        admission: ServiceWorkAdmission<16>,
        cwd: Option<&Path>,
        cols: u16,
        rows: u16,
        sink: impl Fn(PtyChannelEvent) -> Result<(), ()> + Send + 'static,
    ) -> Result<(Self, SessionControl), String> {
        let (session, output) = PtySession::spawn(cwd, cols, rows)?;
        let pid = session.process_id().ok_or_else(terminal_error)?;
        let status = session.child_status();
        let reader_cancelled = Arc::new(AtomicBool::new(false));
        let reader_finished = Arc::new(AtomicBool::new(false));
        let output_window = Arc::new(OutputWindow::new());
        let emergency_stop: Arc<dyn EmergencyStop> = Arc::new(ProcessEmergencyStop {
            pid,
            stopped: AtomicBool::new(false),
        });
        let reader = spawn_reader(
            output,
            Arc::clone(&reader_cancelled),
            Arc::clone(&reader_finished),
            Arc::clone(&output_window),
            Arc::clone(&emergency_stop),
            Box::new(sink),
            Some(Box::new(move || PtyChannelEvent {
                data: String::new(),
                is_exit: true,
                exit_code: status.exit_code(),
                sequence: None,
            })),
        )?;
        let control = SessionControl {
            output_window,
            reader_cancelled,
            reader_finished,
            emergency_stop,
        };
        Ok((
            Self {
                session,
                reader,
                _admission: admission,
            },
            control,
        ))
    }
}

impl SessionOps for OwnedSession {
    fn write(&self, data: &[u8]) -> Result<(), String> {
        self.session.write(data)
    }

    fn resize(&self, cols: u16, rows: u16) -> Result<(), String> {
        self.session.resize(cols, rows)
    }

    fn finish_close(self: Box<Self>) {
        let OwnedSession {
            mut session,
            reader,
            _admission,
        } = *self;
        let _ = session.shutdown();
        drop(session);
        if reader.join().is_err() {
            ::log::warn!("[terminal] lecteur PTY interrompu");
        }
        drop(_admission);
    }

    #[cfg(test)]
    fn process_id(&self) -> Option<u32> {
        self.session.process_id()
    }
}

#[cfg(test)]
pub(super) fn spawn_reader_for_test(
    output: Box<dyn Read + Send>,
    cancelled: Arc<AtomicBool>,
    sink: impl Fn(PtyChannelEvent) -> Result<(), ()> + Send + 'static,
) -> (Arc<AtomicBool>, JoinHandle<()>) {
    let finished = Arc::new(AtomicBool::new(false));
    let reader = spawn_reader(
        output,
        cancelled,
        Arc::clone(&finished),
        Arc::new(OutputWindow::new()),
        Arc::new(super::session_handle::NoopEmergencyStop),
        Box::new(sink),
        None,
    )
    .expect("test reader thread");
    (finished, reader)
}

#[cfg(test)]
pub(super) fn spawn_reader_with_exit_for_test(
    output: Box<dyn Read + Send>,
    sink: impl Fn(PtyChannelEvent) -> Result<(), ()> + Send + 'static,
    exit_code: Option<u32>,
) -> (Arc<AtomicBool>, JoinHandle<()>) {
    let finished = Arc::new(AtomicBool::new(false));
    let reader = spawn_reader(
        output,
        Arc::new(AtomicBool::new(false)),
        Arc::clone(&finished),
        Arc::new(OutputWindow::new()),
        Arc::new(super::session_handle::NoopEmergencyStop),
        Box::new(sink),
        Some(Box::new(move || PtyChannelEvent {
            data: String::new(),
            is_exit: true,
            exit_code,
            sequence: None,
        })),
    )
    .expect("test reader thread");
    (finished, reader)
}
