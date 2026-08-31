use super::manager::PtyManager;
use super::output_window::OutputWindow;
use super::owned_session::spawn_reader_for_test;
use super::session_handle::{EmergencyStop, SessionControl, SessionHandle, SessionOps};
use crate::app_exit::AppExitCoordinator;
use std::io::{Cursor, Read};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{mpsc, Arc, Barrier, Mutex};
use std::time::Duration;

const TOKEN: &str = "0123456789abcdef0123456789abcdef";

fn token() -> zeroize::Zeroizing<String> {
    zeroize::Zeroizing::new(TOKEN.to_string())
}

struct GateOps {
    entered: mpsc::SyncSender<()>,
    release: Mutex<mpsc::Receiver<()>>,
    active: Arc<AtomicUsize>,
    maximum: Arc<AtomicUsize>,
    finishes: Arc<AtomicUsize>,
}

impl SessionOps for GateOps {
    fn write(&self, _data: &[u8]) -> Result<(), String> {
        let active = self.active.fetch_add(1, Ordering::AcqRel) + 1;
        self.maximum.fetch_max(active, Ordering::AcqRel);
        self.entered.send(()).expect("report operation entry");
        self.release
            .lock()
            .unwrap()
            .recv()
            .expect("release operation");
        self.active.fetch_sub(1, Ordering::AcqRel);
        Ok(())
    }

    fn resize(&self, _cols: u16, _rows: u16) -> Result<(), String> {
        self.write(&[])
    }

    fn finish_close(self: Box<Self>) {
        self.finishes.fetch_add(1, Ordering::AcqRel);
    }
}

struct TestStop {
    release: Mutex<Option<mpsc::SyncSender<()>>>,
    stops: AtomicUsize,
}

impl EmergencyStop for TestStop {
    fn stop(&self) {
        self.stops.fetch_add(1, Ordering::AcqRel);
        if let Some(release) = self.release.lock().unwrap().take() {
            release.send(()).expect("emergency release");
        }
    }
}

struct Fixture {
    ops: Box<dyn SessionOps>,
    control: SessionControl,
    entered: mpsc::Receiver<()>,
    release: mpsc::SyncSender<()>,
    stop: Arc<TestStop>,
    maximum: Arc<AtomicUsize>,
    finishes: Arc<AtomicUsize>,
}

fn fixture(stop_releases: bool) -> Fixture {
    let (entered_tx, entered) = mpsc::sync_channel(2);
    let (release, release_rx) = mpsc::sync_channel(2);
    let maximum = Arc::new(AtomicUsize::new(0));
    let finishes = Arc::new(AtomicUsize::new(0));
    let stop = Arc::new(TestStop {
        release: Mutex::new(stop_releases.then(|| release.clone())),
        stops: AtomicUsize::new(0),
    });
    let control = SessionControl {
        output_window: Arc::new(OutputWindow::new()),
        reader_cancelled: Arc::new(AtomicBool::new(false)),
        reader_finished: Arc::new(AtomicBool::new(false)),
        emergency_stop: Arc::clone(&stop) as Arc<dyn EmergencyStop>,
    };
    Fixture {
        ops: Box::new(GateOps {
            entered: entered_tx,
            release: Mutex::new(release_rx),
            active: Arc::new(AtomicUsize::new(0)),
            maximum: Arc::clone(&maximum),
            finishes: Arc::clone(&finishes),
        }),
        control,
        entered,
        release,
        stop,
        maximum,
        finishes,
    }
}

#[test]
fn blocked_write_releases_map_and_kill_stops_before_waiting_for_operations() {
    let coordinator = AppExitCoordinator::initialize().unwrap();
    let manager = PtyManager::new(coordinator.work_supervisor());
    let f = fixture(true);
    let stop = Arc::clone(&f.stop);
    let finishes = Arc::clone(&f.finishes);
    let reader_finished = Arc::clone(&f.control.reader_finished);
    let output_window = Arc::clone(&f.control.output_window);
    let (id, token) = manager.insert_session_for_test(f.ops, f.control, TOKEN);
    assert_eq!(
        manager.write(id, "incorrect", b"forbidden"),
        Err("terminal-access-denied".to_string())
    );
    let writer = {
        let manager = manager.clone();
        let token = token.clone();
        std::thread::spawn(move || manager.write(id, &token, b"blocked"))
    };
    f.entered.recv_timeout(Duration::from_secs(1)).unwrap();
    assert!(manager.manager_lock_is_available_for_test());
    assert!(!reader_finished.load(Ordering::Acquire));
    let _ = output_window.acknowledge(1);
    let (done, result) = mpsc::sync_channel(1);
    let killer = std::thread::spawn(move || done.send(manager.kill(id, &token)).unwrap());
    assert_eq!(result.recv_timeout(Duration::from_secs(3)), Ok(Ok(())));
    writer.join().unwrap().unwrap();
    killer.join().unwrap();
    assert_eq!(stop.stops.load(Ordering::Acquire), 1);
    assert_eq!(finishes.load(Ordering::Acquire), 1);
}

#[test]
fn operations_serialize_per_handle_but_not_between_handles() {
    let first = fixture(false);
    let handle = Arc::new(SessionHandle::new(first.ops, first.control, token()));
    let one = {
        let handle = Arc::clone(&handle);
        std::thread::spawn(move || handle.with_live(|ops| ops.write(b"one")))
    };
    first.entered.recv_timeout(Duration::from_secs(1)).unwrap();
    let two = {
        let handle = Arc::clone(&handle);
        std::thread::spawn(move || handle.with_live(|ops| ops.resize(80, 24)))
    };
    assert!(first
        .entered
        .recv_timeout(Duration::from_millis(100))
        .is_err());
    first.release.send(()).unwrap();
    first.entered.recv_timeout(Duration::from_secs(1)).unwrap();
    first.release.send(()).unwrap();
    one.join().unwrap().unwrap();
    two.join().unwrap().unwrap();
    assert_eq!(first.maximum.load(Ordering::Acquire), 1);

    let left = fixture(false);
    let right = fixture(false);
    let left_handle = Arc::new(SessionHandle::new(left.ops, left.control, token()));
    let right_handle = Arc::new(SessionHandle::new(right.ops, right.control, token()));
    let left_thread = std::thread::spawn(move || left_handle.with_live(|ops| ops.write(b"l")));
    let right_thread = std::thread::spawn(move || right_handle.with_live(|ops| ops.write(b"r")));
    left.entered.recv_timeout(Duration::from_secs(1)).unwrap();
    right.entered.recv_timeout(Duration::from_secs(1)).unwrap();
    left.release.send(()).unwrap();
    right.release.send(()).unwrap();
    left_thread.join().unwrap().unwrap();
    right_thread.join().unwrap().unwrap();
}

#[test]
fn close_is_once_and_control_paths_ignore_the_operations_mutex() {
    let f = fixture(false);
    let finished = Arc::clone(&f.control.reader_finished);
    let window = Arc::clone(&f.control.output_window);
    let finishes = Arc::clone(&f.finishes);
    let handle = Arc::new(SessionHandle::new(f.ops, f.control, token()));
    let barrier = Arc::new(Barrier::new(3));
    let threads = (0..2)
        .map(|_| {
            let handle = Arc::clone(&handle);
            let barrier = Arc::clone(&barrier);
            std::thread::spawn(move || {
                barrier.wait();
                handle.close();
            })
        })
        .collect::<Vec<_>>();
    barrier.wait();
    for thread in threads {
        thread.join().unwrap();
    }
    assert_eq!(finishes.load(Ordering::Acquire), 1);
    assert_eq!(
        handle.with_live(|ops| ops.write(b"late")),
        Err("terminal-not-found".into())
    );
    assert!(!finished.load(Ordering::Acquire));
    let _ = window.acknowledge(1);
}

struct GateReader(mpsc::SyncSender<()>, mpsc::Receiver<()>);

impl Read for GateReader {
    fn read(&mut self, _buf: &mut [u8]) -> std::io::Result<usize> {
        self.0.send(()).unwrap();
        self.1.recv().unwrap();
        Ok(0)
    }
}

#[test]
fn reader_guard_marks_natural_cancelled_and_sink_failure_exits() {
    let (finished, reader) = spawn_reader_for_test(
        Box::new(Cursor::new(Vec::<u8>::new())),
        Arc::new(AtomicBool::new(false)),
        |_| Ok(()),
    );
    reader.join().unwrap();
    assert!(finished.load(Ordering::Acquire));

    let (entered_tx, entered) = mpsc::sync_channel(1);
    let (release, release_rx) = mpsc::sync_channel(1);
    let cancelled = Arc::new(AtomicBool::new(false));
    let (finished, reader) = spawn_reader_for_test(
        Box::new(GateReader(entered_tx, release_rx)),
        Arc::clone(&cancelled),
        |_| Ok(()),
    );
    entered.recv_timeout(Duration::from_secs(1)).unwrap();
    cancelled.store(true, Ordering::Release);
    release.send(()).unwrap();
    reader.join().unwrap();
    assert!(finished.load(Ordering::Acquire));

    let (finished, reader) = spawn_reader_for_test(
        Box::new(Cursor::new(b"data".to_vec())),
        Arc::new(AtomicBool::new(false)),
        |_| Err(()),
    );
    reader.join().unwrap();
    assert!(finished.load(Ordering::Acquire));
}
