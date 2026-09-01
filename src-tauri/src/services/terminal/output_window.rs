use super::limits::{MAX_FRAME_BYTES, MAX_IN_FLIGHT_BYTES, MAX_IN_FLIGHT_FRAMES};
use super::public_error::{not_found, terminal_error};
use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Condvar, Mutex};

struct OutputState {
    closed: bool,
    next_sequence: u32,
    in_flight_bytes: usize,
    frames: VecDeque<(u32, usize)>,
}

pub(super) struct OutputWindow {
    state: Mutex<OutputState>,
    changed: Condvar,
}

impl OutputWindow {
    pub(super) fn new() -> Self {
        Self {
            state: Mutex::new(OutputState {
                closed: false,
                next_sequence: 1,
                in_flight_bytes: 0,
                frames: VecDeque::with_capacity(MAX_IN_FLIGHT_FRAMES),
            }),
            changed: Condvar::new(),
        }
    }

    pub(super) fn reserve(&self, bytes: usize, cancelled: &AtomicBool) -> Result<u32, String> {
        if bytes > MAX_FRAME_BYTES {
            return Err(terminal_error());
        }
        let mut state = self.state.lock().map_err(|_| terminal_error())?;
        loop {
            if cancelled.load(Ordering::Acquire) {
                state.closed = true;
                self.changed.notify_all();
                return Err(not_found());
            }
            if state.closed {
                return Err(not_found());
            }
            let in_flight_bytes = state
                .in_flight_bytes
                .checked_add(bytes)
                .ok_or_else(terminal_error)?;
            if in_flight_bytes <= MAX_IN_FLIGHT_BYTES && state.frames.len() < MAX_IN_FLIGHT_FRAMES {
                let sequence = state.next_sequence;
                let Some(next_sequence) = sequence.checked_add(1) else {
                    state.closed = true;
                    self.changed.notify_all();
                    return Err(not_found());
                };
                state.next_sequence = next_sequence;
                state.in_flight_bytes = in_flight_bytes;
                state.frames.push_back((sequence, bytes));
                return Ok(sequence);
            }
            state = self.changed.wait(state).map_err(|_| terminal_error())?;
        }
    }

    pub(super) fn acknowledge(&self, sequence: u32) -> Result<(), String> {
        let mut state = self.state.lock().map_err(|_| terminal_error())?;
        if state.closed {
            return Err(not_found());
        }
        let position = state
            .frames
            .iter()
            .position(|(candidate, _)| *candidate == sequence)
            .ok_or_else(terminal_error)?;
        let acknowledged_bytes = state
            .frames
            .iter()
            .take(position + 1)
            .try_fold(0_usize, |total, (_, bytes)| {
                total.checked_add(*bytes).ok_or_else(terminal_error)
            })?;
        state.in_flight_bytes = state
            .in_flight_bytes
            .checked_sub(acknowledged_bytes)
            .ok_or_else(terminal_error)?;
        state.frames.drain(..=position);
        self.changed.notify_all();
        Ok(())
    }

    pub(super) fn close(&self) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        state.closed = true;
        self.changed.notify_all();
    }

    #[cfg(test)]
    pub(super) fn outstanding_for_test(&self) -> Result<(usize, usize), String> {
        let state = self.state.lock().map_err(|_| terminal_error())?;
        Ok((state.frames.len(), state.in_flight_bytes))
    }
}
