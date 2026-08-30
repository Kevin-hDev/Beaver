use std::sync::{LazyLock, Mutex};
use std::time::Instant;

use tauri::Emitter;

use crate::models::compression_profile_contract::{
    BudgetProjectionView, CompressionDeleteResult, CompressionProfileInput, CompressionProfilesView,
};
use crate::services::compress::profile_store;
use crate::services::compress::profile_types::CompressionWindowBand;

use super::compression_profiles_undo::{UndoSlot, UNDO_DURATION};

const CHANGED_EVENT: &str = "fs:compression-profiles-changed";
const ERROR_CODE: &str = "compression_profiles_unavailable";
static DELETE_UNDO: LazyLock<Mutex<UndoSlot>> = LazyLock::new(|| Mutex::new(UndoSlot::default()));

#[tauri::command]
pub fn get_compression_profiles() -> Result<CompressionProfilesView, String> {
    profile_store::load_document()
        .map(|document| CompressionProfilesView::from(&document))
        .map_err(map_error)
}

#[tauri::command]
pub fn create_compression_profile(
    app: tauri::AppHandle,
    source_profile_id: String,
    name: String,
) -> Result<CompressionProfilesView, String> {
    mutate_and_emit(&app, |document| {
        super::compression_profiles_mutations::create(document, &source_profile_id, name)
    })
}

#[tauri::command]
pub fn rename_compression_profile(
    app: tauri::AppHandle,
    profile_id: String,
    name: String,
) -> Result<CompressionProfilesView, String> {
    mutate_and_emit(&app, |document| {
        super::compression_profiles_mutations::rename(document, &profile_id, name)
    })
}

#[tauri::command]
pub fn save_compression_profile(
    app: tauri::AppHandle,
    input: CompressionProfileInput,
) -> Result<CompressionProfilesView, String> {
    mutate_and_emit(&app, |document| {
        super::compression_profiles_mutations::save(document, input)
    })
}

#[tauri::command]
pub fn select_global_compression_profile(
    app: tauri::AppHandle,
    profile_id: String,
) -> Result<CompressionProfilesView, String> {
    mutate_and_emit(&app, |document| {
        super::compression_profiles_mutations::select_global(document, &profile_id)
    })
}

#[tauri::command]
pub fn reset_beaver_compression_profile(
    app: tauri::AppHandle,
) -> Result<CompressionProfilesView, String> {
    mutate_and_emit(&app, super::compression_profiles_mutations::reset_beaver)
}

#[tauri::command]
pub fn delete_compression_profile(
    app: tauri::AppHandle,
    profile_id: String,
) -> Result<CompressionDeleteResult, String> {
    let ((before, ()), after) = profile_store::mutate_document(|document| {
        let before = document.clone();
        super::compression_profiles_mutations::delete(document, &profile_id)?;
        Ok((before, ()))
    })
    .map_err(map_error)?;
    let token = DELETE_UNDO
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .record(before, after.clone(), Instant::now());
    emit_changed(&app);
    Ok(CompressionDeleteResult {
        view: CompressionProfilesView::from(&after),
        undo_token: token,
        undo_expires_in_ms: UNDO_DURATION.as_millis() as u32,
    })
}

#[tauri::command]
pub fn undo_delete_compression_profile(
    app: tauri::AppHandle,
    undo_token: String,
) -> Result<CompressionProfilesView, String> {
    let (before, after) = DELETE_UNDO
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .candidate(&undo_token, Instant::now())
        .map_err(map_error)?;
    let (_, restored) = profile_store::mutate_document(|document| {
        if *document != after {
            return Err(profile_store::CompressionProfileStoreError::Invalid);
        }
        *document = before;
        Ok(())
    })
    .map_err(map_error)?;
    DELETE_UNDO
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .clear_if_token(&undo_token);
    emit_changed(&app);
    Ok(CompressionProfilesView::from(&restored))
}

#[tauri::command]
pub fn project_settings_compression_budget(
    profile_id: String,
    band: CompressionWindowBand,
    context_window: u64,
) -> Result<BudgetProjectionView, String> {
    let document = profile_store::load_document().map_err(map_error)?;
    let profile = document
        .profiles
        .iter()
        .find(|profile| profile.id == profile_id)
        .ok_or_else(|| ERROR_CODE.to_string())?;
    super::compression_profiles_projection::project(profile, band, context_window)
        .map_err(map_error)
}

fn mutate_and_emit(
    app: &tauri::AppHandle,
    mutation: impl FnOnce(
        &mut crate::services::compress::profile_store_document::CompressionProfileDocument,
    ) -> Result<(), profile_store::CompressionProfileStoreError>,
) -> Result<CompressionProfilesView, String> {
    let (_, document) = profile_store::mutate_document(mutation).map_err(map_error)?;
    emit_changed(app);
    Ok(CompressionProfilesView::from(&document))
}

fn emit_changed(app: &tauri::AppHandle) {
    if app.emit(CHANGED_EVENT, ()).is_err() {
        log::warn!("compression_profile_event_emit_failed");
    }
}

fn map_error(error: profile_store::CompressionProfileStoreError) -> String {
    log::warn!("compression_profile_command_failed error={error:?}");
    ERROR_CODE.to_string()
}
