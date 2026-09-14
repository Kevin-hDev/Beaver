use super::*;

#[test]
fn maps_every_stage_and_preserves_channel_statuses() {
    let cases = [
        (
            OllamaProgressStage::Preparing,
            UpdateOperationPhase::Preparing,
            "preparing",
        ),
        (
            OllamaProgressStage::Downloading,
            UpdateOperationPhase::Downloading,
            "downloading",
        ),
        (
            OllamaProgressStage::Verifying,
            UpdateOperationPhase::Verifying,
            "verifying",
        ),
        (
            OllamaProgressStage::Validating,
            UpdateOperationPhase::Verifying,
            "validating",
        ),
        (
            OllamaProgressStage::Extracting,
            UpdateOperationPhase::Extracting,
            "extracting",
        ),
        (
            OllamaProgressStage::Committing,
            UpdateOperationPhase::Installing,
            "committing",
        ),
        (
            OllamaProgressStage::Starting,
            UpdateOperationPhase::Restarting,
            "starting",
        ),
        (
            OllamaProgressStage::Recovering,
            UpdateOperationPhase::Recovering,
            "recovering",
        ),
        (
            OllamaProgressStage::RollingBack,
            UpdateOperationPhase::RollingBack,
            "rolling_back",
        ),
        (
            OllamaProgressStage::Cleaning,
            UpdateOperationPhase::Cleaning,
            "cleaning",
        ),
    ];
    for (stage, phase, channel_status) in cases {
        let update = OllamaProgressUpdate {
            stage,
            completed: 4,
            total: 10,
        };
        assert_eq!(operation("id", "Ollama", &update).phase, phase);
        assert_eq!(progress_status(stage), channel_status);
    }
}

#[test]
fn committing_disables_cancellation_and_recovery_is_indeterminate() {
    let committing = operation(
        "id",
        "Ollama",
        &OllamaProgressUpdate {
            stage: OllamaProgressStage::Committing,
            completed: 0,
            total: 0,
        },
    );
    let recovering = operation(
        "id",
        "Ollama",
        &OllamaProgressUpdate {
            stage: OllamaProgressStage::Recovering,
            completed: 4,
            total: 10,
        },
    );
    assert!(!committing.can_cancel);
    assert_eq!(recovering.progress_mode, UpdateProgressMode::Indeterminate);
    assert_eq!(recovering.percent, None);
}
