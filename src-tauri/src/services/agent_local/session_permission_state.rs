use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PermissionFamily {
    Chat,
    Tools,
}

impl PermissionFamily {
    pub fn allows(self, mode: PermissionMode) -> bool {
        matches!(
            (self, mode),
            (Self::Chat, PermissionMode::Chat)
                | (Self::Tools, PermissionMode::Manual | PermissionMode::Auto)
        )
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PermissionMode {
    Chat,
    Manual,
    Auto,
}

impl PermissionMode {
    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "chat" => Ok(Self::Chat),
            "manual" => Ok(Self::Manual),
            "auto" => Ok(Self::Auto),
            _ => Err("Mode d'autorisation invalide".into()),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Chat => "chat",
            Self::Manual => "manual",
            Self::Auto => "auto",
        }
    }

    fn family(self) -> PermissionFamily {
        match self {
            Self::Chat => PermissionFamily::Chat,
            Self::Manual | Self::Auto => PermissionFamily::Tools,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SessionPermissionState {
    pub permission_family: Option<PermissionFamily>,
    pub permission_mode: PermissionMode,
}

pub async fn load(session_id: &str) -> Result<SessionPermissionState, String> {
    super::session_store::get(session_id)
        .await
        .map_err(|_| "Session indisponible".to_string())?;
    if let Some(state) = load_sidecar(session_id).await? {
        return Ok(state);
    }
    let default = super::agent_settings::get_permission_mode().await;
    Ok(SessionPermissionState {
        permission_family: None,
        permission_mode: PermissionMode::parse(&default).unwrap_or(PermissionMode::Auto),
    })
}

pub async fn set_mode(session_id: &str, mode: PermissionMode) -> Result<SessionPermissionState, String> {
    let lock = super::session_store::lock_session(session_id).await;
    let _guard = lock.lock().await;
    let mut state = load(session_id).await?;
    if state.permission_family.is_some_and(|family| !family.allows(mode)) {
        return Err("Ce mode n'est pas disponible pour cette session".into());
    }
    state.permission_mode = mode;
    persist(session_id, &state).await?;
    Ok(state)
}

pub async fn prepare_send(session_id: &str, requested: Option<&str>) -> Result<String, String> {
    let lock = super::session_store::lock_session(session_id).await;
    let _guard = lock.lock().await;
    let mut state = load(session_id).await?;
    let mode = requested
        .map(PermissionMode::parse)
        .transpose()?
        .unwrap_or(state.permission_mode);
    if state.permission_family.is_some_and(|family| !family.allows(mode)) {
        return Err("Ce mode n'est pas disponible pour cette session".into());
    }
    state.permission_family.get_or_insert_with(|| mode.family());
    state.permission_mode = mode;
    persist(session_id, &state).await?;
    Ok(mode.as_str().to_string())
}

pub async fn merge_into_serialized(session_id: &str, value: &mut serde_json::Value) {
    let Ok(Some(state)) = load_sidecar(session_id).await else {
        return;
    };
    value["permission_family"] = serde_json::to_value(state.permission_family).unwrap_or_default();
    value["permission_mode"] = serde_json::to_value(state.permission_mode).unwrap_or_default();
}

pub async fn remove(session_id: &str) {
    if let Ok(path) = sidecar_path(session_id) {
        let _ = tokio::fs::remove_file(path).await;
    }
}

async fn persist(session_id: &str, state: &SessionPermissionState) -> Result<(), String> {
    let target = sidecar_path(session_id)?;
    let dir = target.parent().ok_or_else(generic_error)?;
    tokio::fs::create_dir_all(dir).await.map_err(|_| generic_error())?;
    let tmp = dir.join(format!(".{session_id}.{}.tmp", uuid::Uuid::new_v4()));
    let data = serde_json::to_vec_pretty(state).map_err(|_| generic_error())?;
    tokio::fs::write(&tmp, data).await.map_err(|_| generic_error())?;
    tokio::fs::rename(&tmp, target).await.map_err(|_| generic_error())?;
    let session = super::session_store::get(session_id).await?;
    super::session_store::save(&session).await
}

async fn load_sidecar(session_id: &str) -> Result<Option<SessionPermissionState>, String> {
    let path = sidecar_path(session_id)?;
    let data = match tokio::fs::read(path).await {
        Ok(data) => data,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(_) => return Err(generic_error()),
    };
    serde_json::from_slice(&data).map(Some).map_err(|_| generic_error())
}

fn sidecar_path(session_id: &str) -> Result<PathBuf, String> {
    super::session_store::validate_session_id(session_id)?;
    Ok(crate::services::paths::data_dir()
        .join("session-permissions")
        .join(format!("{session_id}.json")))
}

fn generic_error() -> String {
    "Mise à jour du mode impossible".to_string()
}
