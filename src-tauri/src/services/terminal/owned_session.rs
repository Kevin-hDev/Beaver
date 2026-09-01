use super::manager::PtyChannelEvent;
use super::output_window::OutputWindow;
use super::pty_session::PtySession;
use super::session_handle::{EmergencyStop, SessionControl, SessionOps};
use super::utf8_decoder::Utf8StreamDecoder;
use crate::services::work_registry::ServiceWorkAdmission;
use std::io::Read;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::JoinHandle;

type EventSink = Box<dyn Fn(PtyChannelEvent) -> Result<(), ()> + Send + 'static>;
type ExitEvent = Box<dyn FnOnce() -> PtyChannelEvent + Send + 'static>;

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

struct ReaderFinishedGuard(Arc<AtomicBool>);

impl Drop for ReaderFinishedGuard {
    fn drop(&mut self) {
        // Release publishes reader cleanup before SessionHandle observes the
        // completion with Acquire and removes this session from the manager.
        self.0.store(true, Ordering::Release);
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
        let reader = spawn_reader(
            output,
            Arc::clone(&reader_cancelled),
            Arc::clone(&reader_finished),
            Box::new(sink),
            Some(Box::new(move || PtyChannelEvent {
                data: String::new(),
                is_exit: true,
                exit_code: status.exit_code(),
            })),
        )?;
        let control = SessionControl {
            output_window: Arc::new(OutputWindow::new()),
            reader_cancelled,
            reader_finished,
            emergency_stop: Arc::new(ProcessEmergencyStop {
                pid,
                stopped: AtomicBool::new(false),
            }),
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

fn spawn_reader(
    output: Box<dyn Read + Send>,
    cancelled: Arc<AtomicBool>,
    finished: Arc<AtomicBool>,
    sink: EventSink,
    exit_event: Option<ExitEvent>,
) -> Result<JoinHandle<()>, String> {
    std::thread::Builder::new()
        .name("beaver-pty-reader".to_string())
        .spawn(move || reader_loop(output, cancelled, finished, sink, exit_event))
        .map_err(|_| terminal_error())
}

fn reader_loop(
    mut output: Box<dyn Read + Send>,
    cancelled: Arc<AtomicBool>,
    finished: Arc<AtomicBool>,
    sink: EventSink,
    exit_event: Option<ExitEvent>,
) {
    let _finished = ReaderFinishedGuard(finished);
    let mut buffer = [0_u8; 4096];
    let mut decoder = Utf8StreamDecoder::new();
    loop {
        if cancelled.load(Ordering::Acquire) {
            break;
        }
        match output.read(&mut buffer) {
            Ok(0) | Err(_) => break,
            Ok(read) if !cancelled.load(Ordering::Acquire) => {
                let data = decoder.push(&buffer[..read]);
                if data.is_empty() {
                    continue;
                }
                if sink(PtyChannelEvent {
                    data,
                    is_exit: false,
                    exit_code: None,
                })
                .is_err()
                {
                    break;
                }
            }
            Ok(_) => break,
        }
    }
    if !cancelled.load(Ordering::Acquire) {
        let final_data = decoder.finish();
        if !final_data.is_empty()
            && sink(PtyChannelEvent {
                data: final_data,
                is_exit: false,
                exit_code: None,
            })
            .is_err()
        {
            return;
        }
        if let Some(exit_event) = exit_event {
            let _ = sink(exit_event());
        }
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
        Box::new(sink),
        Some(Box::new(move || PtyChannelEvent {
            data: String::new(),
            is_exit: true,
            exit_code,
        })),
    )
    .expect("test reader thread");
    (finished, reader)
}

fn terminal_error() -> String {
    "terminal-error".to_string()
}
