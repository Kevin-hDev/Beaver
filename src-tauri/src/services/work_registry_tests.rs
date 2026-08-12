use super::work_registry::{
    ServiceWorkAdmissionError, ServiceWorkPhase, ServiceWorkSupervisor, WorkRegistry,
};
use crate::app_exit::{AppExitCoordinator, AppWorkAdmissionError, AppWorkSupervisor};
use std::future;
use std::time::{Duration, Instant};

fn supervisor() -> (AppExitCoordinator, crate::app_exit::AppWorkSupervisor) {
    let coordinator = AppExitCoordinator::initialize().expect("exit coordinator");
    let supervisor = coordinator.work_supervisor();
    (coordinator, supervisor)
}

fn available_app_slots(supervisor: &AppWorkSupervisor) -> usize {
    let mut admissions = Vec::new();
    loop {
        match supervisor.try_admit() {
            Ok(admission) => admissions.push(admission),
            Err(AppWorkAdmissionError::Capacity) => return admissions.len(),
            Err(AppWorkAdmissionError::Closing) => panic!("app admission unexpectedly closed"),
        }
    }
}

async fn wait_until_inactive<const CAPACITY: usize>(registry: &WorkRegistry<CAPACITY>) {
    tokio::time::timeout(Duration::from_secs(1), async {
        while registry.diagnostics().active != 0 {
            tokio::task::yield_now().await;
        }
    })
    .await
    .expect("service work released its slot");
}

#[test]
fn fixed_slots_track_capacity_generations_and_safe_reuse() {
    let (_coordinator, app) = supervisor();
    let registry = WorkRegistry::<2>::new();
    assert_eq!(registry.phase(), ServiceWorkPhase::Open);

    let first = registry.try_admit(&app).expect("first local slot");
    let stale_key = first.key_for_test();
    let second = registry.try_admit(&app).expect("second local slot");
    let error = registry
        .try_admit(&app)
        .expect_err("fixed registry must reject its third admission");
    assert_eq!(error, ServiceWorkAdmissionError::Capacity);
    assert_eq!(error.public_code(), "service-work-capacity-reached");
    assert_eq!(
        registry.diagnostics(),
        super::work_registry::ServiceWorkDiagnostics {
            active: 2,
            high_water: 2,
            saturation_refusals: 1,
            closing_refusals: 0,
        }
    );

    drop(first);
    let reused = registry.try_admit(&app).expect("released slot is reusable");
    let reused_key = reused.key_for_test();
    assert_eq!(reused_key.index, stale_key.index);
    assert_ne!(reused_key.generation, stale_key.generation);
    assert!(!registry.release_key_for_test(stale_key));
    assert_eq!(registry.diagnostics().active, 2);

    drop((second, reused));
    assert_eq!(registry.diagnostics().active, 0);
}

#[tokio::test]
async fn stop_moves_open_to_closing_to_closed_and_is_idempotent() {
    let (_coordinator, app) = supervisor();
    let registry = WorkRegistry::<1>::new();
    let admission = registry.try_admit(&app).expect("tracked service work");
    let cancellation = admission.cancellation();
    let stopping_registry = registry.clone();
    let stop = tokio::spawn(async move {
        stopping_registry
            .stop_and_wait(Instant::now() + Duration::from_secs(1))
            .await
    });

    tokio::time::timeout(Duration::from_secs(1), cancellation.cancelled())
        .await
        .expect("local cancellation");
    assert!(cancellation.is_cancelled());
    assert_eq!(registry.phase(), ServiceWorkPhase::Closing);
    let error = registry
        .try_admit(&app)
        .expect_err("closing service must reject new work");
    assert_eq!(error, ServiceWorkAdmissionError::Closing);
    assert_eq!(error.public_code(), "service-shutting-down");
    assert_eq!(registry.diagnostics().closing_refusals, 1);

    drop(admission);
    assert!(stop.await.expect("stop task"));
    assert_eq!(registry.phase(), ServiceWorkPhase::Closed);
    assert!(
        registry
            .stop_and_wait(Instant::now() + Duration::from_millis(50))
            .await
    );
}

#[tokio::test]
async fn managed_work_releases_slots_on_success_error_panic_and_drop() {
    let (_coordinator, app) = supervisor();
    let registry = WorkRegistry::<1>::new();

    registry
        .spawn(&app, |_| async {})
        .expect("successful work admitted");
    wait_until_inactive(&registry).await;

    registry
        .spawn(&app, |_| async { Err::<(), &'static str>("expected") })
        .expect("failing work admitted");
    wait_until_inactive(&registry).await;

    registry
        .spawn(&app, |_| async { panic!("expected task panic") })
        .expect("panicking work admitted");
    wait_until_inactive(&registry).await;

    drop(registry.try_admit(&app).expect("abandoned admission"));
    assert_eq!(registry.diagnostics().active, 0);
    assert_eq!(registry.diagnostics().high_water, 1);
}

#[tokio::test]
async fn stop_cancels_cooperative_work_and_aborts_uncooperative_work() {
    let (_coordinator, app) = supervisor();
    let cooperative = WorkRegistry::<1>::new();
    cooperative
        .spawn(&app, |cancel| async move { cancel.cancelled().await })
        .expect("cooperative work admitted");
    assert!(
        cooperative
            .stop_and_wait(Instant::now() + Duration::from_secs(1))
            .await
    );
    assert_eq!(cooperative.phase(), ServiceWorkPhase::Closed);
    assert_eq!(cooperative.diagnostics().active, 0);

    let uncooperative = WorkRegistry::<1>::new();
    uncooperative
        .spawn(&app, |_| future::pending::<()>())
        .expect("uncooperative work admitted");
    assert!(
        uncooperative
            .stop_and_wait(Instant::now() + Duration::from_millis(20))
            .await
    );
    assert_eq!(uncooperative.phase(), ServiceWorkPhase::Closed);
    assert_eq!(uncooperative.diagnostics().active, 0);

    let (_baseline_coordinator, baseline_app) = supervisor();
    assert_eq!(
        available_app_slots(&app),
        available_app_slots(&baseline_app),
        "forced local release must synchronously return its global admission"
    );
}

#[tokio::test]
async fn abandoning_stop_still_aborts_every_extracted_handle() {
    let (_coordinator, app) = supervisor();
    let registry = WorkRegistry::<1>::new();
    registry
        .spawn(&app, |_| future::pending::<()>())
        .expect("uncooperative work admitted");
    let stopping_registry = registry.clone();
    let stop = tokio::spawn(async move {
        stopping_registry
            .stop_and_wait(Instant::now() + Duration::from_secs(1))
            .await
    });
    tokio::time::timeout(Duration::from_secs(1), async {
        while registry.phase() != ServiceWorkPhase::Closing {
            tokio::task::yield_now().await;
        }
    })
    .await
    .expect("stop entered its wait phase");

    stop.abort();
    assert!(stop
        .await
        .expect_err("stop must be abandoned")
        .is_cancelled());
    assert_eq!(registry.phase(), ServiceWorkPhase::Closed);
    assert_eq!(registry.diagnostics().active, 0);
}

#[tokio::test]
async fn service_owner_binds_global_and_local_admission_once() {
    let (_coordinator, app) = supervisor();
    let service = ServiceWorkSupervisor::<1>::new(app);
    let admission = service.try_admit().expect("service admission");
    admission
        .spawn(|cancel| async move { cancel.cancelled().await })
        .expect("admitted task starts");

    assert!(
        service
            .stop_and_wait(Instant::now() + Duration::from_secs(1))
            .await
    );
    assert_eq!(service.phase(), ServiceWorkPhase::Closed);
    assert_eq!(service.diagnostics().active, 0);
    assert_eq!(
        service
            .try_admit()
            .expect_err("closed service refuses work"),
        ServiceWorkAdmissionError::Closing
    );
}

#[test]
fn service_probe_checks_both_gates_without_consuming_a_local_slot() {
    let (_coordinator, app) = supervisor();
    let service = ServiceWorkSupervisor::<1>::new(app);

    service.try_probe().expect("open service probe");
    assert_eq!(service.diagnostics().active, 0);
    service.begin_closing();
    assert_eq!(
        service.try_probe().expect_err("closed service probe"),
        ServiceWorkAdmissionError::Closing
    );
    assert_eq!(service.diagnostics().closing_refusals, 1);
}
