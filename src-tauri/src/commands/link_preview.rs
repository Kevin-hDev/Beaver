use crate::services::config as config_service;
use crate::services::link_preview::{self, LinkPreview};

#[tauri::command]
pub async fn fetch_link_preview(url: String) -> Result<LinkPreview, String> {
    let config = config_service::read_config().map_err(|_| "Preview unavailable".to_string())?;
    if !config.advanced.link_preview_enabled {
        return Err("Preview disabled".to_string());
    }
    link_preview::fetch_preview(&url).await
}
