use super::{
    contracts::{VoiceAction, VoiceDeliveryOutcome, VoiceDestination, VoiceRecoveryState},
    runtime::{new_voice_runtime, VoiceRuntime},
};

fn runtime() -> VoiceRuntime {
    let exit = crate::app_exit::AppExitCoordinator::initialize().unwrap();
    new_voice_runtime(exit.work_supervisor())
}

fn start(runtime: &VoiceRuntime, key: &str) -> String {
    runtime
        .dispatch(
            VoiceAction::Start {
                destination: VoiceDestination::Draft {
                    draft_key: key.into(),
                },
                context_generation: 1,
                language: None,
            },
            true,
        )
        .unwrap()
        .operation
        .unwrap()
        .id
}

#[test]
fn cancellation_threshold_and_repeated_cross_are_deterministic() {
    for (speech_ms, recovers) in [(29_999, false), (30_000, true)] {
        let runtime = runtime();
        let operation_id = start(&runtime, "draft");
        runtime
            .coordinator_for_test()
            .record_capture(&operation_id, 40_000, speech_ms, false, 0.5)
            .unwrap();
        runtime
            .dispatch(
                VoiceAction::CancelInsertion {
                    operation_id: operation_id.clone(),
                },
                true,
            )
            .unwrap();
        let first = runtime.snapshot().recovery.map(|item| item.id);
        assert_eq!(first.is_some(), recovers);
        runtime
            .dispatch(
                VoiceAction::CancelInsertion {
                    operation_id: operation_id.clone(),
                },
                true,
            )
            .unwrap();
        assert_eq!(runtime.snapshot().recovery.map(|item| item.id), first);
        runtime
            .coordinator_for_test()
            .complete(&operation_id, "dictée".into(), 100)
            .unwrap();
        assert_eq!(runtime.snapshot().recovery.is_some(), recovers);
    }
}

#[test]
fn delivery_can_be_replayed_then_acknowledged_once() {
    let runtime = runtime();
    let operation_id = start(&runtime, "draft");
    runtime
        .coordinator_for_test()
        .record_capture(&operation_id, 40_000, 35_000, true, 0.5)
        .unwrap();
    runtime
        .coordinator_for_test()
        .complete(&operation_id, "résultat".into(), 100)
        .unwrap();
    let first = runtime.snapshot().delivery.unwrap();
    assert_eq!(runtime.snapshot().delivery.unwrap(), first);
    runtime
        .dispatch(
            VoiceAction::AcknowledgeDelivery {
                result_id: first.id.clone(),
                outcome: VoiceDeliveryOutcome::Inserted,
            },
            true,
        )
        .unwrap();
    runtime
        .dispatch(
            VoiceAction::AcknowledgeDelivery {
                result_id: first.id.clone(),
                outcome: VoiceDeliveryOutcome::Inserted,
            },
            true,
        )
        .unwrap();
    assert!(runtime.snapshot().delivery.is_none());
}

#[test]
fn closing_before_insertion_redirects_a_long_result_to_recovery() {
    let runtime = runtime();
    let operation_id = start(&runtime, "draft");
    runtime
        .coordinator_for_test()
        .record_capture(&operation_id, 40_000, 30_000, false, 0.5)
        .unwrap();
    runtime
        .coordinator_for_test()
        .complete(&operation_id, "résultat".into(), 100)
        .unwrap();
    runtime
        .dispatch(VoiceAction::CancelInsertion { operation_id }, true)
        .unwrap();
    let recovery = runtime.snapshot().recovery.unwrap();
    assert_eq!(recovery.status, VoiceRecoveryState::Ready);
    assert_eq!(recovery.draft_key.as_deref(), Some("draft"));
}

#[test]
fn oversized_recovery_fails_without_truncating_or_replacing_text() {
    let runtime = runtime();
    let operation_id = start(&runtime, "draft");
    runtime
        .coordinator_for_test()
        .record_capture(&operation_id, 40_000, 30_000, false, 0.5)
        .unwrap();
    runtime
        .dispatch(
            VoiceAction::CancelInsertion {
                operation_id: operation_id.clone(),
            },
            true,
        )
        .unwrap();
    runtime
        .coordinator_for_test()
        .complete(&operation_id, "𐐷".repeat(100_001), 100)
        .unwrap();
    let snapshot = runtime.snapshot();
    assert_eq!(
        snapshot.recovery.unwrap().status,
        VoiceRecoveryState::Failed
    );
    assert!(snapshot.delivery.is_none());
    assert!(snapshot.error.is_some());
}

#[test]
fn recovery_moves_global_on_close_and_is_consumed_only_after_acknowledgement() {
    let runtime = runtime();
    let operation_id = start(&runtime, "draft");
    runtime
        .coordinator_for_test()
        .record_capture(&operation_id, 40_000, 30_000, false, 0.5)
        .unwrap();
    runtime
        .dispatch(
            VoiceAction::CancelInsertion {
                operation_id: operation_id.clone(),
            },
            true,
        )
        .unwrap();
    runtime
        .coordinator_for_test()
        .complete(&operation_id, "résultat".into(), 100)
        .unwrap();
    runtime
        .dispatch(
            VoiceAction::DestinationClosed {
                destination: VoiceDestination::Draft {
                    draft_key: "draft".into(),
                },
            },
            true,
        )
        .unwrap();
    let recovery = runtime.snapshot().recovery.unwrap();
    assert!(recovery.draft_key.is_none());
    let delivery = runtime
        .dispatch(
            VoiceAction::RestoreRecovery {
                recovery_id: recovery.id,
                draft_key: "active".into(),
            },
            true,
        )
        .unwrap()
        .delivery
        .unwrap();
    assert_eq!(
        runtime.snapshot().delivery.as_ref().unwrap().text,
        "résultat"
    );
    runtime
        .dispatch(
            VoiceAction::AcknowledgeDelivery {
                result_id: delivery.id.clone(),
                outcome: VoiceDeliveryOutcome::AlreadyInserted,
            },
            true,
        )
        .unwrap();
    assert!(runtime.snapshot().delivery.is_none());
}

#[test]
fn trial_result_is_separate_and_closing_it_never_creates_recovery() {
    let runtime = runtime();
    let operation_id = runtime
        .dispatch(
            VoiceAction::Start {
                destination: VoiceDestination::Trial {
                    trial_id: "trial".into(),
                },
                context_generation: 1,
                language: None,
            },
            true,
        )
        .unwrap()
        .operation
        .unwrap()
        .id;
    runtime
        .coordinator_for_test()
        .record_capture(&operation_id, 40_000, 35_000, false, 0.5)
        .unwrap();
    runtime
        .coordinator_for_test()
        .complete(&operation_id, "essai".into(), 100)
        .unwrap();
    assert_eq!(runtime.snapshot().trial_result.unwrap().text, "essai");
    let snapshot = runtime
        .dispatch(
            VoiceAction::DestinationClosed {
                destination: VoiceDestination::Trial {
                    trial_id: "trial".into(),
                },
            },
            true,
        )
        .unwrap();
    assert!(snapshot.trial_result.is_none());
    assert!(snapshot.recovery.is_none());
}
