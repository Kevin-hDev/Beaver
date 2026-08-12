use super::work_supervision::{McpWorkServices, MAX_MCP_PROCESSES};
use crate::app_exit::AppExitCoordinator;
use crate::services::work_registry::ServiceWorkAdmissionError;
use std::time::{Duration, Instant};

#[test]
fn process_registry_is_bounded_by_the_pool_capacity() {
    let coordinator = AppExitCoordinator::initialize().expect("exit coordinator");
    let work = McpWorkServices::new(coordinator.work_supervisor());
    let admissions = (0..MAX_MCP_PROCESSES)
        .map(|_| work.try_admit_process().expect("process admission"))
        .collect::<Vec<_>>();

    assert_eq!(
        work.try_admit_process()
            .expect_err("process registry must remain bounded"),
        ServiceWorkAdmissionError::Capacity
    );
    drop(admissions);
}

#[tokio::test]
async fn app_closing_cancels_process_admission_and_refuses_restart() {
    let coordinator = AppExitCoordinator::initialize().expect("exit coordinator");
    let work = McpWorkServices::new(coordinator.work_supervisor());
    let admission = work.try_admit_process().expect("process admission");
    let cancellation = admission.cancellation();

    coordinator.close_work_admission_for_test();

    assert!(cancellation.is_cancelled());
    assert_eq!(
        work.try_admit_process()
            .expect_err("closing must refuse MCP restart"),
        ServiceWorkAdmissionError::AppClosing
    );
    drop(admission);
    assert!(
        work.stop_and_wait(Instant::now() + Duration::from_secs(1))
            .await
    );
}
