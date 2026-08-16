use super::{OwnedProcessError, OwnedProcessIdentity, OwnedProcessInspection};
use std::sync::atomic::{AtomicBool, Ordering};

fn identity() -> OwnedProcessIdentity {
    OwnedProcessIdentity {
        pid: 42,
        native_scope: 42,
        native_start_time: 200,
        executable: 7,
    }
}

#[test]
fn reused_pid_is_certainly_unowned_before_executable_inspection() {
    let identity_read = AtomicBool::new(false);

    let inspected = super::platform::recovery::inspect_for_recovery_with(
        42,
        100,
        |_| Some(200),
        |_| {
            identity_read.store(true, Ordering::Release);
            Ok(identity())
        },
    );

    assert_eq!(inspected, Ok(OwnedProcessInspection::Unowned));
    assert!(!identity_read.load(Ordering::Acquire));
}

#[test]
fn matching_start_time_still_requires_the_complete_identity() {
    assert_eq!(
        super::platform::recovery::inspect_for_recovery_with(
            42,
            200,
            |_| Some(200),
            |_| Ok(identity()),
        ),
        Ok(OwnedProcessInspection::Owned(identity()))
    );
    assert_eq!(
        super::platform::recovery::inspect_for_recovery_with(
            42,
            200,
            |_| Some(200),
            |_| Err(OwnedProcessError::Admission),
        ),
        Err(OwnedProcessError::Admission)
    );
}
