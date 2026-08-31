use super::output_window::OutputWindow;
use std::sync::{mpsc, Arc};
use std::time::Duration;

#[test]
fn close_wakes_a_waiter_with_not_found() {
    let window = Arc::new(OutputWindow::new());
    let (entered, observed) = mpsc::sync_channel(1);
    let waiter = {
        let window = Arc::clone(&window);
        std::thread::spawn(move || {
            entered.send(()).expect("report waiter");
            window.wait_until_closed_for_test()
        })
    };
    observed
        .recv_timeout(Duration::from_secs(1))
        .expect("waiter entered");
    window.close();
    assert_eq!(
        waiter.join().expect("waiter thread"),
        Err("terminal-not-found".to_string())
    );
}
