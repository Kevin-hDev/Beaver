use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
struct SchedulerRuntimeState {
    last_checked_at: String,
}

fn state_path() -> PathBuf {
    crate::services::paths::data_dir().join("heartbeat-runtime.json")
}

pub async fn read_last_checked() -> Option<DateTime<Local>> {
    let content = tokio::fs::read_to_string(state_path()).await.ok()?;
    let state = serde_json::from_str::<SchedulerRuntimeState>(&content).ok()?;
    DateTime::parse_from_rfc3339(&state.last_checked_at)
        .ok()
        .map(|dt| dt.with_timezone(&Local))
}

pub async fn write_last_checked(value: DateTime<Local>) -> Result<(), String> {
    let path = state_path();
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|_| "Erreur état scheduler".to_string())?;
    }
    let state = SchedulerRuntimeState {
        last_checked_at: value.to_rfc3339(),
    };
    let tmp = path.with_extension("json.tmp");
    let content =
        serde_json::to_string_pretty(&state).map_err(|_| "Erreur état scheduler".to_string())?;
    tokio::fs::write(&tmp, content)
        .await
        .map_err(|_| "Erreur état scheduler".to_string())?;
    tokio::fs::rename(&tmp, &path)
        .await
        .map_err(|_| "Erreur état scheduler".to_string())?;
    Ok(())
}
