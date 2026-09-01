use super::limits::{MAX_FRAME_BYTES, MAX_IN_FLIGHT_BYTES, MAX_IN_FLIGHT_FRAMES};
use super::output_window::OutputWindow;
use std::sync::atomic::AtomicBool;
use std::sync::{mpsc, Arc};
use std::time::Duration;

fn active_reader() -> Arc<AtomicBool> {
    Arc::new(AtomicBool::new(false))
}

fn blocked_reservation(
    window: &Arc<OutputWindow>,
    bytes: usize,
) -> (
    mpsc::Receiver<()>,
    mpsc::Receiver<Result<u32, String>>,
    std::thread::JoinHandle<()>,
) {
    let (entered_tx, entered_rx) = mpsc::sync_channel(1);
    let (result_tx, result_rx) = mpsc::sync_channel(1);
    let window = Arc::clone(window);
    let waiter = std::thread::spawn(move || {
        entered_tx.send(()).expect("report reservation start");
        let result = window.reserve(bytes, &active_reader());
        result_tx.send(result).expect("report reservation result");
    });
    (entered_rx, result_rx, waiter)
}

fn assert_waiting(entered: &mpsc::Receiver<()>, result: &mpsc::Receiver<Result<u32, String>>) {
    entered
        .recv_timeout(Duration::from_secs(1))
        .expect("reservation started");
    assert_eq!(
        result.recv_timeout(Duration::from_millis(50)),
        Err(mpsc::RecvTimeoutError::Timeout)
    );
}

#[test]
fn byte_limit_blocks_until_the_first_sequence_is_acknowledged() {
    let window = Arc::new(OutputWindow::new());
    let cancelled = active_reader();
    let first = window.reserve(MAX_FRAME_BYTES, &cancelled).unwrap();
    for _ in 1..(MAX_IN_FLIGHT_BYTES / MAX_FRAME_BYTES) {
        window.reserve(MAX_FRAME_BYTES, &cancelled).unwrap();
    }
    let (entered, result, waiter) = blocked_reservation(&window, 1);
    assert_waiting(&entered, &result);

    window.acknowledge(first).unwrap();

    assert_eq!(result.recv_timeout(Duration::from_secs(1)).unwrap(), Ok(17));
    waiter.join().unwrap();
}

#[test]
fn frame_limit_blocks_the_257th_frame_below_the_byte_limit() {
    let window = Arc::new(OutputWindow::new());
    let cancelled = active_reader();
    let first = window.reserve(1, &cancelled).unwrap();
    for _ in 1..MAX_IN_FLIGHT_FRAMES {
        window.reserve(1, &cancelled).unwrap();
    }
    let (entered, result, waiter) = blocked_reservation(&window, 1);
    assert_waiting(&entered, &result);

    window.acknowledge(first).unwrap();

    assert_eq!(
        result.recv_timeout(Duration::from_secs(1)).unwrap(),
        Ok(257)
    );
    waiter.join().unwrap();
}

#[test]
fn acknowledgement_is_cumulative_and_never_underflows() {
    let window = OutputWindow::new();
    let cancelled = active_reader();
    window.reserve(3, &cancelled).unwrap();
    let second = window.reserve(5, &cancelled).unwrap();
    window.reserve(7, &cancelled).unwrap();

    window.acknowledge(second).unwrap();

    assert_eq!(window.outstanding_for_test().unwrap(), (1, 7));
    window.acknowledge(3).unwrap();
    assert_eq!(window.outstanding_for_test().unwrap(), (0, 0));
}

#[test]
fn future_and_unknown_acknowledgements_are_rejected_without_mutation() {
    let window = OutputWindow::new();
    let cancelled = active_reader();
    let first = window.reserve(8, &cancelled).unwrap();

    assert_eq!(window.acknowledge(first + 1), Err("terminal-error".into()));
    assert_eq!(window.outstanding_for_test().unwrap(), (1, 8));
    window.acknowledge(first).unwrap();
    assert_eq!(window.acknowledge(first), Err("terminal-error".into()));
    assert_eq!(window.outstanding_for_test().unwrap(), (0, 0));
}

#[test]
fn close_wakes_all_waiters_with_not_found() {
    let window = Arc::new(OutputWindow::new());
    let cancelled = active_reader();
    for _ in 0..MAX_IN_FLIGHT_FRAMES {
        window.reserve(1, &cancelled).unwrap();
    }
    let (first_entered, first_result, first_waiter) = blocked_reservation(&window, 1);
    let (second_entered, second_result, second_waiter) = blocked_reservation(&window, 1);
    assert_waiting(&first_entered, &first_result);
    assert_waiting(&second_entered, &second_result);

    window.close();

    for result in [first_result, second_result] {
        assert_eq!(
            result.recv_timeout(Duration::from_secs(1)).unwrap(),
            Err("terminal-not-found".into())
        );
    }
    first_waiter.join().unwrap();
    second_waiter.join().unwrap();
}

#[test]
fn oversized_frame_is_rejected_before_window_mutation() {
    let window = OutputWindow::new();
    let cancelled = active_reader();

    assert_eq!(
        window.reserve(MAX_FRAME_BYTES + 1, &cancelled),
        Err("terminal-error".into())
    );
    assert_eq!(window.outstanding_for_test().unwrap(), (0, 0));
    assert_eq!(window.reserve(1, &cancelled), Ok(1));
}
