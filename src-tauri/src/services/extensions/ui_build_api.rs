pub(crate) fn resolve_runtime(
    app: &tauri::AppHandle,
) -> Result<super::ui_builder::UiBuildRuntime, String> {
    super::ui_builder::UiBuildRuntime::resolve(app).map_err(|error| error.code().to_string())
}

pub(crate) fn prepare_record(
    record: &mut super::types::ExtensionRecord,
    runtime: &super::ui_builder::UiBuildRuntime,
) -> Result<(), String> {
    super::ui_builder::prepare_record(record, runtime, || false)
        .map_err(|error| error.code().to_string())
}

pub(crate) async fn refresh_artifacts(app: &tauri::AppHandle) -> Result<Vec<String>, String> {
    super::ui_builder::refresh_all(app).await
}

pub(crate) fn cleanup_unreferenced() -> Result<(), String> {
    super::ui_artifact_store::unreferenced_from_registry()
}
