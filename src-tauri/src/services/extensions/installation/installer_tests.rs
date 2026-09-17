use super::super::OperationFailure;
use super::ensure_uninstall_active;

#[test]
fn cancelled_uninstall_is_refused_before_registry_mutation() {
    let coordinator = crate::app_exit::AppExitCoordinator::initialize().unwrap();
    let work =
        super::super::work_supervision::ExtensionWorkServices::new(coordinator.work_supervisor());
    let admission = work.try_admit_operation().unwrap();
    let cancel = admission.cancellation();
    work.begin_closing();

    assert_eq!(
        ensure_uninstall_active(&cancel),
        Err(OperationFailure::HostUnavailable)
    );
}
