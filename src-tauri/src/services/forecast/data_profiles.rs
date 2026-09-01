use super::data_quality::DataProfile;
use super::limits::{MAX_DATA_PROFILES, MAX_INLINE_DATA_BYTES};
use super::types::ForecastRequest;
use crate::services::workspace_scope::WorkspaceScope;
use std::path::{Path, PathBuf};
use tokio::sync::Mutex;
use uuid::Uuid;

pub use super::data_profiles_load::{
    hydrate_request, hydrate_request_classified, load_profile_classified, DataProfileHydrateError,
    DataProfileLoadError,
};

const MAX_DIRECTORY_SCAN: usize = 1_000;
pub(super) static PROFILE_LOCK: Mutex<()> = Mutex::const_new(());

pub async fn save(
    session_id: &str,
    profile: &DataProfile,
    request: &ForecastRequest,
) -> Result<(), String> {
    let _lifecycle_guard = super::workspace_lifecycle::lock().await;
    validate_id(&profile.id)?;
    if !profile.valid {
        return Err("Profil de données invalide".into());
    }
    let data = request.data.as_deref().ok_or("Données JSON requises")?;
    if data.len() > MAX_INLINE_DATA_BYTES {
        return Err("Données trop volumineuses".into());
    }
    let workspace = crate::services::workspace_scope::resolve(session_id).await?;
    let stored = super::data_profiles_load::StoredDataProfile {
        profile: profile.clone(),
        data: data.to_string(),
        workspace: workspace.clone(),
    };
    let json =
        serde_json::to_vec_pretty(&stored).map_err(|_| "Profil de données invalide".to_string())?;
    let _guard = PROFILE_LOCK.lock().await;
    let target = profile_path_for_write(&workspace, &profile.id).await?;
    let dir = target
        .parent()
        .ok_or("Sauvegarde du profil impossible")?
        .to_path_buf();
    let already_exists = tokio::fs::try_exists(&target)
        .await
        .map_err(|_| "Sauvegarde du profil impossible".to_string())?;
    let keep_before_write = if already_exists {
        MAX_DATA_PROFILES
    } else {
        MAX_DATA_PROFILES.saturating_sub(1)
    };
    cleanup(&dir, keep_before_write).await?;
    crate::services::private_store::atomic_write_async(target, json)
        .await
        .map_err(|_| "Sauvegarde du profil impossible".to_string())
}

async fn cleanup(dir: &Path, keep: usize) -> Result<(), String> {
    let mut entries = match tokio::fs::read_dir(dir).await {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(_) => return Err("Sauvegarde du profil impossible".into()),
    };
    let mut files = Vec::new();
    let mut scanned = 0usize;
    while scanned < MAX_DIRECTORY_SCAN {
        let Some(entry) = entries
            .next_entry()
            .await
            .map_err(|_| "Sauvegarde du profil impossible".to_string())?
        else {
            break;
        };
        scanned += 1;
        if is_profile_file(&entry.path()) {
            let modified = entry
                .metadata()
                .await
                .ok()
                .and_then(|meta| meta.modified().ok());
            files.push((modified, entry.path()));
        }
    }
    if scanned == MAX_DIRECTORY_SCAN
        && entries
            .next_entry()
            .await
            .map_err(|_| "Sauvegarde du profil impossible".to_string())?
            .is_some()
    {
        return Err("Trop de profils de données".into());
    }
    files.sort_by_key(|item| item.0);
    let remove_count = files.len().saturating_sub(keep);
    for (_, path) in files.into_iter().take(remove_count) {
        tokio::fs::remove_file(path)
            .await
            .map_err(|_| "Nettoyage du profil impossible".to_string())?;
    }
    Ok(())
}

pub(super) async fn profile_path_for_read(
    workspace: &WorkspaceScope,
    id: &str,
) -> std::io::Result<PathBuf> {
    crate::services::paths::data_file_for_read(
        &profile_directory(workspace)?,
        &format!("{id}.json"),
    )
    .await
}

pub(super) async fn profile_path_for_write(
    workspace: &WorkspaceScope,
    id: &str,
) -> Result<PathBuf, String> {
    let directory =
        profile_directory(workspace).map_err(|_| "Sauvegarde du profil impossible".to_string())?;
    crate::services::paths::data_file_for_write(&directory, &format!("{id}.json"))
        .await
        .map_err(|_| "Sauvegarde du profil impossible".to_string())
}

pub(super) fn profile_directory(workspace: &WorkspaceScope) -> std::io::Result<String> {
    let value = match workspace {
        WorkspaceScope::Project(id) => format!("forecast-data-profiles-project-{id}"),
        WorkspaceScope::Session(id) => format!("forecast-data-profiles-session-{id}"),
        WorkspaceScope::Legacy => return Err(std::io::ErrorKind::InvalidInput.into()),
    };
    Ok(value)
}

pub(super) async fn legacy_profile_path_for_read(id: &str) -> std::io::Result<PathBuf> {
    crate::services::paths::data_file_for_read("forecast-data-profiles", &format!("{id}.json"))
        .await
}

pub(super) async fn legacy_profile_path_for_write(id: &str) -> Result<PathBuf, String> {
    crate::services::paths::data_file_for_write("forecast-data-profiles", &format!("{id}.json"))
        .await
        .map_err(|_| "Sauvegarde du profil impossible".to_string())
}

fn validate_id(id: &str) -> Result<(), String> {
    Uuid::parse_str(id)
        .map(|_| ())
        .map_err(|_| "Identifiant de profil invalide".into())
}

pub(super) fn is_profile_file(path: &Path) -> bool {
    path.extension().and_then(|value| value.to_str()) == Some("json")
        && path
            .file_stem()
            .and_then(|value| value.to_str())
            .is_some_and(|value| Uuid::parse_str(value).is_ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_accepts_uuid_profile_files() {
        assert!(is_profile_file(Path::new(
            "550e8400-e29b-41d4-a716-446655440000.json"
        )));
        assert!(!is_profile_file(Path::new("../profile.json")));
    }

    #[test]
    fn each_workspace_has_its_own_profile_directory() {
        let project = WorkspaceScope::Project("project-a".into());
        let session = WorkspaceScope::Session("session-a".into());

        assert_ne!(
            profile_directory(&project).unwrap(),
            profile_directory(&session).unwrap()
        );
        assert!(profile_directory(&WorkspaceScope::Legacy).is_err());
    }
}
