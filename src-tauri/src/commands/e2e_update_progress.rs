use crate::services::update_progress::{
    UpdateOperationKind, UpdateOperationPhase, UpdateOperationSnapshot, UpdateOperationStatus,
    UpdateProgressMode, UpdateProgressRuntime,
};

#[tauri::command]
pub async fn e2e_seed_update_operation(
    app: tauri::AppHandle,
    runtime: tauri::State<'_, UpdateProgressRuntime>,
) -> Result<(), String> {
    if runtime
        .snapshot()?
        .iter()
        .any(|operation| operation.id == "e2e-update")
    {
        runtime.finish(
            &app,
            UpdateOperationSnapshot {
                id: "e2e-update".into(),
                sequence: 2,
                kind: UpdateOperationKind::AppRelease,
                label: "Beaver E2E".into(),
                status: UpdateOperationStatus::Completed,
                phase: UpdateOperationPhase::Completed,
                progress_mode: UpdateProgressMode::Determinate,
                percent: Some(100),
                queue_position: None,
                can_cancel: false,
                can_retry: false,
                is_update: None,
                error_key: None,
                missing_bytes: None,
            },
        )?;
        return Ok(());
    }

    let operations = [
        UpdateOperationSnapshot {
            id: "e2e-update".into(),
            sequence: 1,
            kind: UpdateOperationKind::AppRelease,
            label: "Beaver E2E".into(),
            status: UpdateOperationStatus::Running,
            phase: UpdateOperationPhase::Downloading,
            progress_mode: UpdateProgressMode::Determinate,
            percent: Some(25),
            queue_position: None,
            can_cancel: true,
            can_retry: false,
            is_update: None,
            error_key: None,
            missing_bytes: None,
        },
        UpdateOperationSnapshot {
            id: "e2e-ollama".into(),
            sequence: 1,
            kind: UpdateOperationKind::OllamaBinary,
            label: "Ollama E2E".into(),
            status: UpdateOperationStatus::Running,
            phase: UpdateOperationPhase::Verifying,
            progress_mode: UpdateProgressMode::Indeterminate,
            percent: None,
            queue_position: None,
            can_cancel: true,
            can_retry: false,
            is_update: None,
            error_key: None,
            missing_bytes: None,
        },
        UpdateOperationSnapshot {
            id: "e2e-queued".into(),
            sequence: 1,
            kind: UpdateOperationKind::ForecastModel,
            label: "TimesFM E2E".into(),
            status: UpdateOperationStatus::Queued,
            phase: UpdateOperationPhase::Waiting,
            progress_mode: UpdateProgressMode::None,
            percent: None,
            queue_position: Some(2),
            can_cancel: false,
            can_retry: false,
            is_update: Some(false),
            error_key: None,
            missing_bytes: None,
        },
        UpdateOperationSnapshot {
            id: "e2e-cancelling".into(),
            sequence: 1,
            kind: UpdateOperationKind::OllamaModel,
            label: "nomic E2E".into(),
            status: UpdateOperationStatus::Cancelling,
            phase: UpdateOperationPhase::Downloading,
            progress_mode: UpdateProgressMode::Determinate,
            percent: Some(64),
            queue_position: None,
            can_cancel: false,
            can_retry: false,
            is_update: Some(false),
            error_key: None,
            missing_bytes: None,
        },
    ];
    for operation in operations {
        runtime.upsert(&app, operation)?;
    }
    Ok(())
}
