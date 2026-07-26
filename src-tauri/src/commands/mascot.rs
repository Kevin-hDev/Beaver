use crate::models::{MascotSettings, MascotSettingsPatch};
use crate::services::mascot::{self, MascotStatePayload};

#[tauri::command]
pub async fn get_mascot_settings() -> Result<MascotSettings, String> {
    mascot::get_settings().await
}

#[tauri::command]
pub async fn patch_mascot_settings(
    app: tauri::AppHandle,
    patch: MascotSettingsPatch,
) -> Result<MascotSettings, String> {
    mascot::patch_settings(&app, patch).await
}

#[tauri::command]
pub async fn save_mascot_position(app: tauri::AppHandle, x: i32, y: i32) -> Result<(), String> {
    mascot::save_position(&app, x, y).await
}

#[tauri::command]
pub async fn get_mascot_state(app: tauri::AppHandle) -> Result<MascotStatePayload, String> {
    mascot::current_state(&app)
}
