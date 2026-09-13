use super::*;

#[test]
fn app_release_stops_cancellation_before_restart() {
    let preparing = running_operation(
        "id",
        "Beaver 1.2.3",
        UpdateOperationPhase::Preparing,
        UpdateProgressMode::Indeterminate,
        None,
        true,
    );
    let restarting = running_operation(
        "id",
        "Beaver 1.2.3",
        UpdateOperationPhase::Restarting,
        UpdateProgressMode::Indeterminate,
        None,
        false,
    );

    assert_eq!(preparing.kind, UpdateOperationKind::AppRelease);
    assert!(preparing.can_cancel);
    assert!(!restarting.can_cancel);
}

#[test]
fn maps_success_cancellation_and_failure_to_terminal_states() {
    assert_eq!(
        terminal_status(None),
        (UpdateOperationStatus::Completed, None)
    );
    assert_eq!(
        terminal_status(Some("update-download-cancelled")),
        (UpdateOperationStatus::Cancelled, None)
    );
    assert_eq!(
        terminal_status(Some("internal path must stay hidden")),
        (UpdateOperationStatus::Failed, Some("update-download-error"))
    );
}
