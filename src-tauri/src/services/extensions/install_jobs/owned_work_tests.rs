use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, Instant};

#[tokio::test]
async fn abandoned_wait_keeps_blocking_producer_admitted_until_it_stops() {
    let coordinator = crate::app_exit::AppExitCoordinator::initialize().unwrap();
    let work =
        super::super::work_supervision::ExtensionWorkServices::new(coordinator.work_supervisor());
    let gate = Arc::new((Mutex::new(false), Condvar::new()));
    let writer_gate = gate.clone();
    let receiver = super::owned_work::spawn(&work, move || {
        let (lock, changed) = &*writer_gate;
        let mut finished = lock.lock().unwrap();
        while !*finished {
            finished = changed.wait(finished).unwrap();
        }
        7
    })
    .unwrap();
    drop(receiver);
    assert!(
        !work
            .stop_and_wait(Instant::now() + Duration::from_millis(20))
            .await
    );
    assert_eq!(work.operation_diagnostics().active, 1);
    *gate.0.lock().unwrap() = true;
    gate.1.notify_all();
    assert!(
        work.stop_and_wait(Instant::now() + Duration::from_secs(2))
            .await
    );
    assert_eq!(work.operation_diagnostics().active, 0);
}
