use super::output_window::OutputWindow;
use super::public_error::terminal_error;
use super::session_handle::{EmergencyStop, ReaderFinishedGuard};
use super::utf8_decoder::Utf8StreamDecoder;
use super::PtyChannelEvent;
use std::io::Read;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::JoinHandle;

type EventSink = Box<dyn Fn(PtyChannelEvent) -> Result<(), ()> + Send + 'static>;
type ExitEvent = Box<dyn FnOnce() -> PtyChannelEvent + Send + 'static>;

pub(super) fn spawn_reader(
    output: Box<dyn Read + Send>,
    cancelled: Arc<AtomicBool>,
    finished: Arc<AtomicBool>,
    output_window: Arc<OutputWindow>,
    emergency_stop: Arc<dyn EmergencyStop>,
    sink: EventSink,
    exit_event: Option<ExitEvent>,
) -> Result<JoinHandle<()>, String> {
    std::thread::Builder::new()
        .name("beaver-pty-reader".to_string())
        .spawn(move || {
            let _finished = ReaderFinishedGuard(finished);
            let result = catch_unwind(AssertUnwindSafe(|| {
                reader_loop(
                    output,
                    cancelled.as_ref(),
                    output_window.as_ref(),
                    emergency_stop.as_ref(),
                    sink.as_ref(),
                    exit_event,
                )
            }));
            if result.is_err() && !cancelled.load(Ordering::Acquire) {
                output_window.close();
                emergency_stop.stop();
                let _ = send_event(
                    sink.as_ref(),
                    PtyChannelEvent {
                        data: String::new(),
                        is_exit: true,
                        exit_code: None,
                        sequence: None,
                    },
                );
            }
        })
        .map_err(|_| terminal_error())
}

fn reader_loop(
    mut output: Box<dyn Read + Send>,
    cancelled: &AtomicBool,
    output_window: &OutputWindow,
    emergency_stop: &dyn EmergencyStop,
    sink: &(dyn Fn(PtyChannelEvent) -> Result<(), ()> + Send),
    exit_event: Option<ExitEvent>,
) {
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
                if !data.is_empty()
                    && !emit_data(data, cancelled, output_window, emergency_stop, sink)
                {
                    return;
                }
            }
            Ok(_) => break,
        }
    }
    if cancelled.load(Ordering::Acquire) {
        return;
    }
    let final_data = decoder.finish();
    if !final_data.is_empty()
        && !emit_data(final_data, cancelled, output_window, emergency_stop, sink)
    {
        return;
    }
    if let Some(exit_event) = exit_event {
        if !send_event(sink, exit_event()) {
            output_window.close();
            emergency_stop.stop();
        }
    }
}

fn emit_data(
    data: String,
    cancelled: &AtomicBool,
    output_window: &OutputWindow,
    emergency_stop: &dyn EmergencyStop,
    sink: &(dyn Fn(PtyChannelEvent) -> Result<(), ()> + Send),
) -> bool {
    let Ok(sequence) = output_window.reserve(data.len(), cancelled) else {
        emergency_stop.stop();
        return false;
    };
    if send_event(
        sink,
        PtyChannelEvent {
            data,
            is_exit: false,
            exit_code: None,
            sequence: Some(sequence),
        },
    ) {
        true
    } else {
        output_window.close();
        emergency_stop.stop();
        false
    }
}

fn send_event(
    sink: &(dyn Fn(PtyChannelEvent) -> Result<(), ()> + Send),
    event: PtyChannelEvent,
) -> bool {
    matches!(catch_unwind(AssertUnwindSafe(|| sink(event))), Ok(Ok(())))
}
