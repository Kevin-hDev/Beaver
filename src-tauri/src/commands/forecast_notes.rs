use crate::services::forecast::notes;
use tauri::AppHandle;

#[tauri::command]
pub async fn list_forecast_notes(
    app: AppHandle,
    analysis_id: String,
) -> Result<Vec<notes::ForecastNote>, String> {
    let result = notes::list(&analysis_id).await?;
    if let Some(revision) = result.revision {
        crate::services::forecast::events::emit_updated_id(
            &app,
            &analysis_id,
            result.session_id.as_deref(),
            Some(revision),
        );
    }
    Ok(result.notes)
}

#[tauri::command]
pub async fn create_forecast_note(
    app: AppHandle,
    request: notes::ForecastNoteCreateRequest,
) -> Result<notes::ForecastNote, String> {
    let mutation = notes::create(request).await?;
    crate::services::forecast::events::emit_updated_id(
        &app,
        &mutation.value.analysis_id,
        mutation.session_id.as_deref(),
        Some(mutation.revision),
    );
    Ok(mutation.value)
}

#[tauri::command]
pub async fn update_forecast_note(
    app: AppHandle,
    request: notes::ForecastNoteUpdateRequest,
) -> Result<notes::ForecastNote, String> {
    let mutation = notes::update(request).await?;
    crate::services::forecast::events::emit_updated_id(
        &app,
        &mutation.value.analysis_id,
        mutation.session_id.as_deref(),
        Some(mutation.revision),
    );
    Ok(mutation.value)
}

#[tauri::command]
pub async fn delete_forecast_note(
    app: AppHandle,
    analysis_id: String,
    note_id: String,
) -> Result<(), String> {
    let mutation = notes::delete(&analysis_id, &note_id).await?;
    crate::services::forecast::events::emit_updated_id(
        &app,
        &analysis_id,
        mutation.session_id.as_deref(),
        Some(mutation.revision),
    );
    Ok(())
}

#[tauri::command]
pub async fn open_forecast_note(analysis_id: String, note_id: String) -> Result<(), String> {
    notes::open(&analysis_id, &note_id).await
}
