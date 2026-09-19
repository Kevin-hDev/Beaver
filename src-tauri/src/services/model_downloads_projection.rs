use super::model_downloads_types::{
    ModelDownloadKind, ModelDownloadPhase, ModelDownloadState, ModelDownloadStatus,
};
use super::update_progress::{
    UpdateOperationKind, UpdateOperationPhase, UpdateOperationSnapshot, UpdateOperationStatus,
    UpdateProgressMode, UpdateProgressRuntime,
};
use tauri::AppHandle;

pub fn project_model_downloads(
    app: &AppHandle,
    states: &[ModelDownloadState],
    progress: &UpdateProgressRuntime,
) {
    for operation in project_states(states) {
        let _ = if operation.status.is_terminal() {
            progress.finish(app, operation)
        } else {
            progress.upsert(app, operation)
        };
    }
}

pub(crate) fn project_states(states: &[ModelDownloadState]) -> Vec<UpdateOperationSnapshot> {
    let mut queue_position = 0_u8;
    states
        .iter()
        .map(|state| {
            if state.status == ModelDownloadStatus::Queued {
                queue_position = queue_position.saturating_add(1);
            }
            project_state(
                state,
                (state.status == ModelDownloadStatus::Queued).then_some(queue_position),
            )
        })
        .collect()
}

pub(crate) fn project_state(
    state: &ModelDownloadState,
    queue_position: Option<u8>,
) -> UpdateOperationSnapshot {
    let status = match state.status {
        ModelDownloadStatus::Queued => UpdateOperationStatus::Queued,
        ModelDownloadStatus::Running => UpdateOperationStatus::Running,
        ModelDownloadStatus::Cancelling => UpdateOperationStatus::Cancelling,
        ModelDownloadStatus::Completed => UpdateOperationStatus::Completed,
        ModelDownloadStatus::Failed => UpdateOperationStatus::Failed,
        ModelDownloadStatus::Cancelled => UpdateOperationStatus::Cancelled,
        ModelDownloadStatus::Suspended => UpdateOperationStatus::Cancelled,
    };
    let phase = match (state.status, state.phase) {
        (ModelDownloadStatus::Queued, _) => UpdateOperationPhase::Waiting,
        (_, ModelDownloadPhase::Starting | ModelDownloadPhase::PreparingRuntime) => {
            UpdateOperationPhase::Preparing
        }
        (_, ModelDownloadPhase::Downloading) => UpdateOperationPhase::Downloading,
        (_, ModelDownloadPhase::Installing) => UpdateOperationPhase::Installing,
        (_, ModelDownloadPhase::Completed) => UpdateOperationPhase::Completed,
    };
    let (progress_mode, percent) = if state.status == ModelDownloadStatus::Queued {
        (UpdateProgressMode::None, None)
    } else if phase == UpdateOperationPhase::Downloading && state.total > 0 {
        (
            UpdateProgressMode::Determinate,
            Some(state.percent.min(100)),
        )
    } else if status == UpdateOperationStatus::Completed {
        (UpdateProgressMode::Determinate, Some(100))
    } else {
        (UpdateProgressMode::Indeterminate, None)
    };
    let can_cancel = matches!(
        status,
        UpdateOperationStatus::Queued | UpdateOperationStatus::Running
    ) && !(matches!(
        state.kind,
        ModelDownloadKind::Forecast | ModelDownloadKind::Voice
    ) && state.phase == ModelDownloadPhase::Installing);
    UpdateOperationSnapshot {
        id: state.id.clone(),
        sequence: 0,
        kind: match state.kind {
            ModelDownloadKind::Ollama => UpdateOperationKind::OllamaModel,
            ModelDownloadKind::Forecast => UpdateOperationKind::ForecastModel,
            ModelDownloadKind::Voice => UpdateOperationKind::VoiceModel,
        },
        label: state
            .active_model_id
            .as_ref()
            .unwrap_or(&state.model_id)
            .clone(),
        status,
        phase,
        progress_mode,
        percent,
        queue_position,
        can_cancel,
        can_retry: status == UpdateOperationStatus::Failed,
        is_update: Some(state.is_update),
        error_key: state.error_key.clone(),
        missing_bytes: state.missing_bytes,
    }
}

#[cfg(test)]
#[path = "model_downloads_projection_tests.rs"]
mod tests;
