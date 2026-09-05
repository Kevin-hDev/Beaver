pub(crate) async fn refresh_artifacts(app: &tauri::AppHandle) -> Result<Vec<String>, String> {
    super::ui_builder::refresh_all(app).await
}
