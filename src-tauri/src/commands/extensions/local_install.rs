use crate::services::extensions::{self, ExtensionView};

pub(super) async fn install(_app: &tauri::AppHandle, path: &str) -> Result<ExtensionView, String> {
    let record = extensions::install_jobs::global()?
        .wait_install(extensions::install_jobs::InstallRequest::Local { path: path.into() })
        .await
        .map_err(|error| error.code().to_string())?;
    Ok(ExtensionView::from(record))
}
