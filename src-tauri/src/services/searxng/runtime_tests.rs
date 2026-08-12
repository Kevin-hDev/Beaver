use super::*;
use crate::app_exit::AppExitCoordinator;
use crate::services::work_registry::ServiceWorkSupervisor;
use std::time::{Duration, Instant};

#[tokio::test]
async fn cancellation_terminates_a_real_runtime_process() {
    let python = ["python3", "python"]
        .into_iter()
        .find_map(|candidate| which::which(candidate).ok())
        .expect("runtime Python de test");
    let coordinator = AppExitCoordinator::initialize().expect("exit coordinator");
    let supervisor = ServiceWorkSupervisor::<1>::new(coordinator.work_supervisor());
    let admission = supervisor.try_admit().expect("runtime admission");
    let cancel = admission.cancellation();
    let mut command = Command::new(python);
    command.args(["-c", "import time; time.sleep(30)"]);
    let started = Instant::now();

    let close = async {
        tokio::time::sleep(Duration::from_millis(100)).await;
        coordinator.close_work_admission_for_test();
    };
    let ((), result) = tokio::join!(close, run(command, &cancel));

    assert!(result.is_err());
    assert!(started.elapsed() < Duration::from_secs(3));
}
