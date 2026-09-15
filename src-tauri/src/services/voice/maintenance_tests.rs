use super::{
    actions::VoiceCoordinator,
    contracts::VoiceDestination,
    delivery_types::{Delivery, TrialResult},
    recovery::Recovery,
};

#[test]
fn ready_recovery_and_delivery_expire_without_rounding() {
    let mut recovery = Recovery::preparing(
        "recovery".into(),
        &VoiceDestination::Draft {
            draft_key: "draft".into(),
        },
        1,
    );
    assert!(recovery.mark_ready("texte".into(), 120_000));
    assert!(!recovery.is_expired_at(719_999));
    assert!(recovery.is_expired_at(720_000));
}

#[test]
fn abandoned_trial_results_expire_from_memory() {
    let mut coordinator = VoiceCoordinator::default();
    coordinator.trial_result = Some(TrialResult {
        trial_id: "trial".into(),
        text: "texte".into(),
        capture_ms: 1,
        compute_ms: 1,
        created_at_ms: 120_000,
    });
    coordinator.expire_at(719_999);
    assert!(coordinator.snapshot().trial_result.is_some());
    coordinator.expire_at(720_000);
    assert!(coordinator.snapshot().trial_result.is_none());
}

#[test]
fn maintenance_removes_expired_delivery_and_exposes_a_generic_error() {
    let mut coordinator = VoiceCoordinator::default();
    coordinator.delivery = Some(
        Delivery::new(
            "operation".into(),
            "draft".into(),
            "texte".into(),
            120_000,
            1,
            1,
        )
        .unwrap(),
    );
    coordinator.expire_at(719_999);
    assert!(coordinator.snapshot().delivery.is_some());
    coordinator.expire_at(720_000);
    let snapshot = coordinator.snapshot();
    assert!(snapshot.delivery.is_none());
    assert!(snapshot.error.is_some());
}
