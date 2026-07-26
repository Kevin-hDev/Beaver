use super::data_quality::DataProfile;
use super::limits::{MAX_DATA_PROFILES, MAX_INLINE_DATA_BYTES};
use super::types::ForecastRequest;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tokio::io::AsyncReadExt;
use tokio::sync::Mutex;
use uuid::Uuid;

const MAX_DIRECTORY_SCAN: usize = 1_000;
static PROFILE_LOCK: Mutex<()> = Mutex::const_new(());

#[derive(Serialize, Deserialize)]
struct StoredDataProfile {
    profile: DataProfile,
    data: String,
}

pub async fn hydrate_request(request: &mut ForecastRequest) -> Result<(), String> {
    let Some(id) = request.data_profile_id.as_deref() else {
        return Ok(());
    };
    if request.data.is_some() || request.file_path.is_some() {
        return Err("Référence de profil ambiguë".into());
    }
    let stored = load(id).await?;
    if !matches_request(&stored.profile, request) {
        return Err("Profil de données incompatible".into());
    }
    request.data = Some(stored.data);
    Ok(())
}

pub async fn save(profile: &DataProfile, request: &ForecastRequest) -> Result<(), String> {
    validate_id(&profile.id)?;
    if !profile.valid {
        return Err("Profil de données invalide".into());
    }
    let data = request.data.as_deref().ok_or("Données JSON requises")?;
    if data.len() > MAX_INLINE_DATA_BYTES {
        return Err("Données trop volumineuses".into());
    }
    let stored = StoredDataProfile {
        profile: profile.clone(),
        data: data.to_string(),
    };
    let json =
        serde_json::to_vec_pretty(&stored).map_err(|_| "Profil de données invalide".to_string())?;
    let _guard = PROFILE_LOCK.lock().await;
    let target = profile_path_for_write(&profile.id).await?;
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

async fn load(id: &str) -> Result<StoredDataProfile, String> {
    validate_id(id)?;
    let max_bytes = MAX_INLINE_DATA_BYTES.saturating_add(64 * 1024);
    let path =
        crate::services::paths::data_file_for_read("forecast-data-profiles", &format!("{id}.json"))
            .await
            .map_err(|_| "Profil de données introuvable".to_string())?;
    let file = tokio::fs::File::open(path)
        .await
        .map_err(|_| "Profil de données introuvable".to_string())?;
    let mut data = Vec::with_capacity(max_bytes.min(64 * 1024));
    file.take((max_bytes + 1) as u64)
        .read_to_end(&mut data)
        .await
        .map_err(|_| "Profil de données invalide".to_string())?;
    if data.len() > max_bytes {
        return Err("Profil de données invalide".into());
    }
    let mut stored: StoredDataProfile =
        serde_json::from_slice(&data).map_err(|_| "Profil de données invalide".to_string())?;
    if stored.profile.id != id || !stored.profile.valid || stored.data.len() > MAX_INLINE_DATA_BYTES
    {
        return Err("Profil de données invalide".into());
    }
    super::data_profile_migration::ensure_fingerprint(&mut stored.profile, &stored.data);
    Ok(stored)
}

pub async fn load_profile(id: &str) -> Result<DataProfile, String> {
    load(id).await.map(|stored| stored.profile)
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

fn matches_request(profile: &DataProfile, request: &ForecastRequest) -> bool {
    profile.target_column == request.target_column
        && profile.date_column == request.date_column
        && profile.series_column == request.series_column
        && profile.covariate_columns == request.covariate_columns
        && profile.frequency == request.frequency
        && profile.horizon == request.horizon
        && profile
            .confidence_level
            .is_some_and(|confidence| (confidence - request.confidence_level).abs() < 0.000_001)
}

async fn profile_path_for_write(id: &str) -> Result<PathBuf, String> {
    crate::services::paths::data_file_for_write("forecast-data-profiles", &format!("{id}.json"))
        .await
        .map_err(|_| "Sauvegarde du profil impossible".to_string())
}

fn validate_id(id: &str) -> Result<(), String> {
    Uuid::parse_str(id)
        .map(|_| ())
        .map_err(|_| "Identifiant de profil invalide".into())
}

fn is_profile_file(path: &Path) -> bool {
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
}
