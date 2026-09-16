use crate::services::update_progress::{
    UpdateOperationKind, UpdateOperationPhase, UpdateOperationSnapshot, UpdateOperationStatus,
    UpdateProgressMode, UpdateProgressRuntime,
};
use tauri::AppHandle;

pub(super) fn running_operation(
    id: &str,
    label: &str,
    phase: UpdateOperationPhase,
    progress_mode: UpdateProgressMode,
    percent: Option<u8>,
    can_cancel: bool,
) -> UpdateOperationSnapshot {
    UpdateOperationSnapshot {
        id: id.into(),
        sequence: 0,
        kind: UpdateOperationKind::AppRelease,
        label: label.into(),
        status: UpdateOperationStatus::Running,
        phase,
        progress_mode,
        percent,
        queue_position: None,
        can_cancel,
        can_retry: false,
        is_update: None,
        error_key: None,
        missing_bytes: None,
    }
}

pub(super) fn terminal_status(
    error: Option<&str>,
) -> (UpdateOperationStatus, Option<&'static str>) {
    match error {
        None => (UpdateOperationStatus::Completed, None),
        Some("update-download-cancelled") => (UpdateOperationStatus::Cancelled, None),
        Some(_) => (UpdateOperationStatus::Failed, Some("update-download-error")),
    }
}

pub(super) fn finish(
    app: &AppHandle,
    progress: &UpdateProgressRuntime,
    id: &str,
    status: UpdateOperationStatus,
    error_key: Option<&str>,
) -> Result<bool, String> {
    let mut operation = progress
        .snapshot()?
        .into_iter()
        .find(|operation| operation.id == id)
        .ok_or_else(|| "update-progress-not-found".to_string())?;
    operation.status = status;
    operation.can_cancel = false;
    operation.can_retry = status == UpdateOperationStatus::Failed;
    operation.error_key = error_key.map(str::to_string);
    if status == UpdateOperationStatus::Completed {
        operation.phase = UpdateOperationPhase::Completed;
        operation.progress_mode = UpdateProgressMode::Determinate;
        operation.percent = Some(100);
    }
    progress.finish(app, operation)
}

#[cfg(test)]
#[path = "app_update_progress_tests.rs"]
mod tests;
