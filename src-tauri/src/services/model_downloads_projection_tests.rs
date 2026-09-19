use super::*;

fn state(
    kind: ModelDownloadKind,
    status: ModelDownloadStatus,
    phase: ModelDownloadPhase,
) -> ModelDownloadState {
    ModelDownloadState {
        id: "model-1".into(),
        kind,
        model_id: "model".into(),
        active_model_id: None,
        is_update: true,
        status,
        phase,
        percent: 40,
        downloaded: 4,
        total: 10,
        error_key: None,
        missing_bytes: None,
    }
}

#[test]
fn voice_projection_names_the_component_being_downloaded() {
    let mut source = state(
        ModelDownloadKind::Voice,
        ModelDownloadStatus::Running,
        ModelDownloadPhase::Downloading,
    );
    source.model_id = "cohere-transcribe-int8".into();
    source.active_model_id = Some("silero-vad".into());
    assert_eq!(project_state(&source, None).label, "silero-vad");
}

#[test]
fn projects_every_status_without_inventing_progress() {
    let cases = [
        (ModelDownloadStatus::Queued, UpdateOperationStatus::Queued),
        (ModelDownloadStatus::Running, UpdateOperationStatus::Running),
        (
            ModelDownloadStatus::Cancelling,
            UpdateOperationStatus::Cancelling,
        ),
        (
            ModelDownloadStatus::Completed,
            UpdateOperationStatus::Completed,
        ),
        (ModelDownloadStatus::Failed, UpdateOperationStatus::Failed),
        (
            ModelDownloadStatus::Cancelled,
            UpdateOperationStatus::Cancelled,
        ),
    ];
    for (source, expected) in cases {
        let projected = project_state(
            &state(
                ModelDownloadKind::Ollama,
                source,
                ModelDownloadPhase::Downloading,
            ),
            (source == ModelDownloadStatus::Queued).then_some(1),
        );
        assert_eq!(projected.status, expected);
        assert_eq!(
            projected.queue_position,
            (source == ModelDownloadStatus::Queued).then_some(1)
        );
        assert_eq!(projected.is_update, Some(true));
        assert_eq!(
            projected.can_cancel,
            matches!(
                source,
                ModelDownloadStatus::Queued | ModelDownloadStatus::Running
            )
        );
    }
}

#[test]
fn forecast_stops_cancellation_at_installing_but_ollama_does_not() {
    let forecast = project_state(
        &state(
            ModelDownloadKind::Forecast,
            ModelDownloadStatus::Running,
            ModelDownloadPhase::Installing,
        ),
        None,
    );
    let ollama = project_state(
        &state(
            ModelDownloadKind::Ollama,
            ModelDownloadStatus::Running,
            ModelDownloadPhase::Installing,
        ),
        None,
    );
    assert!(!forecast.can_cancel);
    assert!(ollama.can_cancel);
    assert_eq!(forecast.progress_mode, UpdateProgressMode::Indeterminate);
}

#[test]
fn maps_model_phases_to_the_closed_progress_contract() {
    for (source, expected) in [
        (
            ModelDownloadPhase::Starting,
            UpdateOperationPhase::Preparing,
        ),
        (
            ModelDownloadPhase::Downloading,
            UpdateOperationPhase::Downloading,
        ),
        (
            ModelDownloadPhase::PreparingRuntime,
            UpdateOperationPhase::Preparing,
        ),
        (
            ModelDownloadPhase::Installing,
            UpdateOperationPhase::Installing,
        ),
        (
            ModelDownloadPhase::Completed,
            UpdateOperationPhase::Completed,
        ),
    ] {
        assert_eq!(
            project_state(
                &state(
                    ModelDownloadKind::Ollama,
                    ModelDownloadStatus::Running,
                    source
                ),
                None
            )
            .phase,
            expected,
        );
    }
}

#[test]
fn projects_the_exact_voice_disk_shortfall() {
    let mut source = state(
        ModelDownloadKind::Voice,
        ModelDownloadStatus::Failed,
        ModelDownloadPhase::Starting,
    );
    source.missing_bytes = Some(42);
    assert_eq!(project_state(&source, None).missing_bytes, Some(42));
}

#[test]
fn projects_four_simultaneous_operations_in_order() {
    let states = vec![
        state(
            ModelDownloadKind::Ollama,
            ModelDownloadStatus::Running,
            ModelDownloadPhase::Downloading,
        ),
        state(
            ModelDownloadKind::Forecast,
            ModelDownloadStatus::Queued,
            ModelDownloadPhase::Starting,
        ),
        state(
            ModelDownloadKind::Ollama,
            ModelDownloadStatus::Queued,
            ModelDownloadPhase::Starting,
        ),
        state(
            ModelDownloadKind::Forecast,
            ModelDownloadStatus::Failed,
            ModelDownloadPhase::Installing,
        ),
    ];
    let projected = project_states(&states);

    assert_eq!(projected.len(), 4);
    assert_eq!(projected[1].queue_position, Some(1));
    assert_eq!(projected[2].queue_position, Some(2));
    assert_eq!(projected[3].status, UpdateOperationStatus::Failed);
}
