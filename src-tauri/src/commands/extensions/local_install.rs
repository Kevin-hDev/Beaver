use crate::services::extensions::{self, ExtensionView};

pub(super) async fn install(app: &tauri::AppHandle, path: &str) -> Result<ExtensionView, String> {
    let mut extension = extensions::install_local(path)?;
    let runtime = extensions::resolve_ui_build_runtime(app)?;
    extension = tokio::task::spawn_blocking(move || {
        extensions::prepare_ui_record(&mut extension.record, &runtime)?;
        Ok::<_, String>(extension)
    })
    .await
    .map_err(|_| extensions::error_codes::INSTALL_FAILED.to_string())??;
    let view = ExtensionView::from(extension.record.clone());
    if let Err(error) = extensions::add_local(extension.record) {
        let _ = extensions::cleanup_unreferenced_ui_artifacts();
        return Err(error);
    }
    Ok(view)
}
