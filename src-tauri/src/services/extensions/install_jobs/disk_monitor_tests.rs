use super::*;
use std::{
    sync::Mutex,
    time::{Duration, Instant},
};

struct Capture(Arc<Mutex<Option<InstallControl>>>);
impl InstallExecutor for Capture {
    fn execute(&self, request: InstallRequest, control: InstallControl) -> InstallFuture {
        *self.0.lock().unwrap() = Some(control.clone());
        Suspended.execute(request, control)
    }
}
async fn control() -> InstallControl {
    let captured = Arc::new(Mutex::new(None));
    let store = store(Arc::new(Capture(captured.clone())));
    store.start(request("disk-monitor")).unwrap();
    tokio::time::timeout(Duration::from_secs(2), async {
        loop {
            if let Some(control) = captured.lock().unwrap().clone() {
                return control;
            }
            tokio::task::yield_now().await;
        }
    })
    .await
    .unwrap()
}

#[tokio::test]
async fn disk_rest_begins_after_a_slow_scan_and_skips_the_next_scan() {
    let control = control().await;
    control.save(Default::default()).unwrap();
    let interval = control.store.disk_policy.poll_interval;
    control
        .poll_disk_with(false, |_| {
            std::thread::sleep(interval + Duration::from_millis(20));
            Ok((0, u64::MAX))
        })
        .unwrap();
    control
        .poll_disk_with(false, |_| panic!("no rest after slow scan"))
        .unwrap();
    let at = control.store.lock().unwrap().jobs[0]
        .monitor
        .sampled_at
        .unwrap();
    assert!(at.elapsed() < interval);
    stop(&control.store).await;
}

#[tokio::test]
async fn forced_scans_serialize_but_do_not_hold_the_state_lock() {
    use std::sync::Barrier;
    let control = control().await;
    control.save(Default::default()).unwrap();
    let entered = Arc::new(Barrier::new(2));
    let release = Arc::new(Barrier::new(2));
    let completed = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let attempting = Arc::new(Barrier::new(2));
    let (entered_scan, observed_scan) = std::sync::mpsc::channel();
    std::thread::scope(|scope| {
        let first = scope.spawn(|| {
            control.poll_disk_with(true, |_| {
                entered.wait();
                release.wait();
                completed.store(true, std::sync::atomic::Ordering::SeqCst);
                Ok((0, u64::MAX))
            })
        });
        entered.wait();
        let second = scope.spawn(|| {
            attempting.wait();
            control.poll_disk_with(true, |_| {
                entered_scan.send(()).unwrap();
                assert!(completed.load(std::sync::atomic::Ordering::SeqCst));
                Ok((0, u64::MAX))
            })
        });
        attempting.wait();
        let state_available = control.store.state.try_lock().is_ok();
        let overlapping = observed_scan
            .recv_timeout(Duration::from_millis(50))
            .is_ok();
        release.wait();
        first.join().unwrap().unwrap();
        second.join().unwrap().unwrap();
        assert!(state_available);
        assert!(
            !overlapping,
            "a forced traversal overlapped the previous scan"
        );
    });
    stop(&control.store).await;
}

#[tokio::test]
async fn disk_failure_preserves_its_classification_and_releases_the_scan_guard() {
    let control = control().await;
    assert_eq!(control.storage_budget(), Err(InstallInterruption::Failed));
    control.save(Default::default()).unwrap();
    let start = Instant::now();
    assert_eq!(
        control.poll_disk_with(true, |_| Err(InstallInterruption::Failed)),
        Err(InstallInterruption::Failed)
    );
    assert!(
        control.store.lock().unwrap().jobs[0]
            .monitor
            .sampled_at
            .unwrap()
            >= start
    );
    assert_eq!(
        control.poll_disk_with(false, |_| panic!("failed scans also rest")),
        Err(InstallInterruption::Failed)
    );
    assert_eq!(
        control.poll_disk_with(true, |_| Ok((0, u64::MAX))),
        Err(InstallInterruption::Failed)
    );
    control.store.lock().unwrap().jobs[0].checkpoint = None;
    assert_eq!(control.storage_budget(), Err(InstallInterruption::Failed));
    stop(&control.store).await;
}
