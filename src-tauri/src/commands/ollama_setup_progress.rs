use super::ollama_setup::OllamaSetupProgress;
use crate::services::ollama_manager::{
    OllamaProgressReporter, OllamaProgressStage, OllamaProgressUpdate,
};
use crate::services::update_progress::{
    UpdateOperationKind, UpdateOperationPhase, UpdateOperationSnapshot, UpdateOperationStatus,
    UpdateProgressMode, UpdateProgressRuntime,
};
use tauri::{ipc::Channel, AppHandle};

pub(super) fn composite_reporter(
    on_progress: &Channel<OllamaSetupProgress>,
    app: &AppHandle,
    progress: &UpdateProgressRuntime,
    id: &str,
    label: &str,
) -> OllamaProgressReporter {
    let channel = on_progress.clone();
    let app = app.clone();
    let progress = progress.clone();
    let id = id.to_string();
    let label = label.to_string();
    std::sync::Arc::new(move |update: OllamaProgressUpdate| {
        let _ = channel.send(channel_update(&update));
        let _ = progress.upsert(&app, operation(&id, &label, &update));
    })
}

pub(super) fn begin(
    app: &AppHandle,
    progress: &UpdateProgressRuntime,
    id: &str,
    label: &str,
) -> Result<bool, String> {
    progress.upsert(
        app,
        operation(
            id,
            label,
            &OllamaProgressUpdate {
                stage: OllamaProgressStage::Preparing,
                completed: 0,
                total: 0,
            },
        ),
    )
}

pub(super) fn mark_cancelling(
    app: &AppHandle,
    progress: &UpdateProgressRuntime,
) -> Result<(), String> {
    for mut operation in progress.snapshot()? {
        if operation.kind == UpdateOperationKind::OllamaBinary && !operation.status.is_terminal() {
            operation.status = UpdateOperationStatus::Cancelling;
            operation.can_cancel = false;
            progress.upsert(app, operation)?;
        }
    }
    Ok(())
}

pub(super) fn send_stage(reporter: &OllamaProgressReporter, stage: OllamaProgressStage) {
    reporter(OllamaProgressUpdate {
        stage,
        completed: 0,
        total: 0,
    });
}

pub(super) fn finish(
    app: &AppHandle,
    progress: &UpdateProgressRuntime,
    id: &str,
    status: UpdateOperationStatus,
    error_key: Option<&str>,
    can_retry: bool,
) -> Result<bool, String> {
    let mut operation = progress
        .snapshot()?
        .into_iter()
        .find(|operation| operation.id == id)
        .ok_or_else(|| "update-progress-not-found".to_string())?;
    operation.status = status;
    operation.can_cancel = false;
    operation.can_retry = can_retry && status == UpdateOperationStatus::Failed;
    operation.error_key = error_key.map(str::to_string);
    if status == UpdateOperationStatus::Completed {
        operation.phase = UpdateOperationPhase::Completed;
        operation.progress_mode = UpdateProgressMode::Determinate;
        operation.percent = Some(100);
    }
    progress.finish(app, operation)
}

pub(super) fn progress_status(stage: OllamaProgressStage) -> &'static str {
    match stage {
        OllamaProgressStage::Preparing => "preparing",
        OllamaProgressStage::Downloading => "downloading",
        OllamaProgressStage::Verifying => "verifying",
        OllamaProgressStage::Extracting => "extracting",
        OllamaProgressStage::Validating => "validating",
        OllamaProgressStage::Committing => "committing",
        OllamaProgressStage::Starting => "starting",
        OllamaProgressStage::Recovering => "recovering",
        OllamaProgressStage::RollingBack => "rolling_back",
        OllamaProgressStage::Cleaning => "cleaning",
    }
}

fn channel_update(update: &OllamaProgressUpdate) -> OllamaSetupProgress {
    OllamaSetupProgress {
        completed: update.completed,
        total: update.total,
        status: progress_status(update.stage).into(),
    }
}

fn operation(id: &str, label: &str, update: &OllamaProgressUpdate) -> UpdateOperationSnapshot {
    let phase = match update.stage {
        OllamaProgressStage::Preparing => UpdateOperationPhase::Preparing,
        OllamaProgressStage::Downloading => UpdateOperationPhase::Downloading,
        OllamaProgressStage::Verifying | OllamaProgressStage::Validating => {
            UpdateOperationPhase::Verifying
        }
        OllamaProgressStage::Extracting => UpdateOperationPhase::Extracting,
        OllamaProgressStage::Committing => UpdateOperationPhase::Installing,
        OllamaProgressStage::Starting => UpdateOperationPhase::Restarting,
        OllamaProgressStage::Recovering => UpdateOperationPhase::Recovering,
        OllamaProgressStage::RollingBack => UpdateOperationPhase::RollingBack,
        OllamaProgressStage::Cleaning => UpdateOperationPhase::Cleaning,
    };
    let determinate = update.stage == OllamaProgressStage::Downloading && update.total > 0;
    let percent = determinate.then(|| {
        (u128::from(update.completed).saturating_mul(100) / u128::from(update.total)).min(100) as u8
    });
    UpdateOperationSnapshot {
        id: id.into(),
        sequence: 0,
        kind: UpdateOperationKind::OllamaBinary,
        label: label.into(),
        status: UpdateOperationStatus::Running,
        phase,
        progress_mode: if determinate {
            UpdateProgressMode::Determinate
        } else {
            UpdateProgressMode::Indeterminate
        },
        percent,
        queue_position: None,
        can_cancel: matches!(
            update.stage,
            OllamaProgressStage::Preparing
                | OllamaProgressStage::Downloading
                | OllamaProgressStage::Verifying
                | OllamaProgressStage::Extracting
                | OllamaProgressStage::Validating
        ),
        can_retry: false,
        is_update: None,
        error_key: None,
        missing_bytes: None,
    }
}

#[cfg(test)]
#[path = "ollama_setup_progress_tests.rs"]
mod tests;
