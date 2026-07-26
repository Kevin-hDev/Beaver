use super::session_tabs_state::SessionTabs;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::sync::Mutex;
use uuid::Uuid;

pub(super) static TABS_LOCK: Mutex<()> = Mutex::const_new(());

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub(super) struct SessionTabsFile {
    #[serde(default)]
    pub(super) sessions: HashMap<String, SessionTabs>,
}

pub(super) fn tabs_path() -> PathBuf {
    crate::services::paths::data_dir().join("session-tabs.json")
}

pub(super) async fn read_file() -> Result<SessionTabsFile, String> {
    match tokio::fs::read_to_string(tabs_path()).await {
        Ok(data) => serde_json::from_str(&data).map_err(|_| "Fichier d'onglets invalide".into()),
        Err(_) => Ok(SessionTabsFile::default()),
    }
}

pub(super) async fn write_file(file: &SessionTabsFile) -> Result<(), String> {
    let path = tabs_path();
    if let Some(dir) = path.parent() {
        tokio::fs::create_dir_all(dir)
            .await
            .map_err(|e| e.to_string())?;
    }
    let tmp = path.with_file_name(format!(".session-tabs.{}.tmp", Uuid::new_v4()));
    let data = serde_json::to_string_pretty(file).map_err(|e| e.to_string())?;
    tokio::fs::write(&tmp, data)
        .await
        .map_err(|e| e.to_string())?;
    tokio::fs::rename(&tmp, &path)
        .await
        .map_err(|e| e.to_string())
}
