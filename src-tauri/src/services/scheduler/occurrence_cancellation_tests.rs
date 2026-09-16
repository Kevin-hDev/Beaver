use super::occurrence_cancellation::OccurrenceCancellation;
use std::sync::Arc;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

#[test]
fn stopping_extension_cancels_only_owned_wakeups() {
    let registry = Arc::new(OccurrenceCancellation::default());
    let parent = CancellationToken::new();
    let (_first_guard, first) = registry
        .admit(Uuid::new_v4(), Uuid::new_v4(), Some("one"), &parent)
        .unwrap();
    let (_neighbor_guard, neighbor) = registry
        .admit(Uuid::new_v4(), Uuid::new_v4(), Some("two"), &parent)
        .unwrap();
    let (_native_guard, native) = registry
        .admit(Uuid::new_v4(), Uuid::new_v4(), None, &parent)
        .unwrap();

    registry.revoke_owner("one");

    assert!(first.is_cancelled());
    assert!(!neighbor.is_cancelled());
    assert!(!native.is_cancelled());
}

#[test]
fn revocation_racing_admission_cannot_launch_work() {
    let registry = Arc::new(OccurrenceCancellation::default());
    registry.revoke_owner("owner");
    assert!(registry
        .admit(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Some("owner"),
            &CancellationToken::new(),
        )
        .is_err());

    registry.allow_owner("owner");
    let (_guard, cancel) = registry
        .admit(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Some("owner"),
            &CancellationToken::new(),
        )
        .unwrap();
    registry.revoke_owner("owner");
    assert!(cancel.is_cancelled());
}

#[test]
fn parent_stop_cancels_child_token_and_guard_releases_capacity() {
    let registry = Arc::new(OccurrenceCancellation::default());
    let parent = CancellationToken::new();
    let occurrence_id = Uuid::new_v4();
    let (guard, cancel) = registry
        .admit(occurrence_id, Uuid::new_v4(), None, &parent)
        .unwrap();
    parent.cancel();
    assert!(cancel.is_cancelled());
    drop(guard);
    assert!(registry
        .admit(
            occurrence_id,
            Uuid::new_v4(),
            None,
            &CancellationToken::new(),
        )
        .is_ok());
}

#[test]
fn global_pause_cancels_all_admitted_wakeups() {
    let registry = Arc::new(OccurrenceCancellation::default());
    let parent = CancellationToken::new();
    let (_first_guard, first) = registry
        .admit(Uuid::new_v4(), Uuid::new_v4(), Some("owner"), &parent)
        .unwrap();
    let (_second_guard, second) = registry
        .admit(Uuid::new_v4(), Uuid::new_v4(), None, &parent)
        .unwrap();
    registry.set_globally_paused(true);
    assert!(first.is_cancelled());
    assert!(second.is_cancelled());
    assert!(registry
        .admit(
            Uuid::new_v4(),
            Uuid::new_v4(),
            None,
            &CancellationToken::new(),
        )
        .is_err());
    registry.set_globally_paused(false);
    assert!(registry
        .admit(
            Uuid::new_v4(),
            Uuid::new_v4(),
            None,
            &CancellationToken::new(),
        )
        .is_ok());
}

#[test]
fn failed_disable_blocks_only_that_automation_until_a_successful_retry() {
    let registry = Arc::new(OccurrenceCancellation::default());
    let blocked = Uuid::new_v4();
    let neighbor = Uuid::new_v4();
    registry.block_automation(blocked);

    assert!(registry
        .admit(
            Uuid::new_v4(),
            blocked,
            Some("owner"),
            &CancellationToken::new(),
        )
        .is_err());
    assert!(registry
        .admit(
            Uuid::new_v4(),
            neighbor,
            Some("owner"),
            &CancellationToken::new(),
        )
        .is_ok());

    registry.allow_automation(blocked);
    assert!(registry
        .admit(
            Uuid::new_v4(),
            blocked,
            Some("owner"),
            &CancellationToken::new(),
        )
        .is_ok());
}
