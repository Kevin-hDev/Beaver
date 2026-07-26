use crate::services::forecast::limits::MAX_STORED_ANALYSIS_BYTES;
use crate::services::forecast::types::{ForecastAnalysisMeta, ForecastResult};
use std::path::{Path, PathBuf};
use tokio::sync::Mutex;

use super::storage_paths::{
    analysis_path_for_read, analysis_path_for_write, validate_analysis_id, validate_analysis_name,
};

static SAVE_LOCK: Mutex<()> = Mutex::const_new(());

pub async fn save(result: &mut ForecastResult) -> Result<(), String> {
    validate_analysis_id(&result.id)?;
    result.name = validate_analysis_name(&result.name)?;
    validate_session_id(result.session_id.as_deref())?;
    let _save_guard = SAVE_LOCK.lock().await;
    let target = analysis_path_for_write(&result.id).await?;
    let previous = read_existing(&target).await?;
    let stored_revision = previous
        .as_deref()
        .map(|bytes| {
            serde_json::from_slice::<ForecastResult>(bytes)
                .map(|stored| stored.revision)
                .map_err(|_| "Données d'analyse corrompues".to_string())
        })
        .transpose()?;
    let incoming_revision = result.revision;
    result.revision = super::storage_revision::next(stored_revision, incoming_revision)?;
    let json = match serde_json::to_vec_pretty(result) {
        Ok(json) => json,
        Err(_) => {
            result.revision = incoming_revision;
            return Err("Erreur de sérialisation".into());
        }
    };
    if json.len() > MAX_STORED_ANALYSIS_BYTES {
        result.revision = incoming_revision;
        return Err("Analyse Forecast trop volumineuse".into());
    }
    if crate::services::private_store::atomic_write_async(target.clone(), json)
        .await
        .is_err()
    {
        result.revision = incoming_revision;
        return Err("Erreur de sauvegarde".into());
    }

    let evicted = match super::storage_index::upsert(result.to_meta()).await {
        Ok(evicted) => evicted,
        Err(error) => {
            result.revision = incoming_revision;
            if restore_previous(target, previous).await.is_err() {
                return Err("Erreur de sauvegarde".into());
            }
            return Err(error);
        }
    };
    cleanup_evicted(evicted).await;
    Ok(())
}

pub async fn load(id: &str) -> Result<ForecastResult, String> {
    validate_analysis_id(id)?;
    let path = analysis_path_for_read(id)
        .await
        .map_err(|_| "Analyse introuvable".to_string())?;
    let data = super::storage_io::read_bounded(&path, MAX_STORED_ANALYSIS_BYTES)
        .await
        .map_err(|_| "Analyse introuvable".to_string())?;
    serde_json::from_slice(&data).map_err(|_| "Données d'analyse corrompues".to_string())
}

pub async fn exists(id: &str) -> Result<bool, String> {
    validate_analysis_id(id)?;
    match analysis_path_for_read(id).await {
        Ok(_) => Ok(true),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(_) => Err("Accès à l'analyse impossible".into()),
    }
}

pub async fn delete(id: &str) -> Result<(), String> {
    validate_analysis_id(id)?;
    let _save_guard = SAVE_LOCK.lock().await;
    let target = analysis_path_for_write(id).await?;
    let previous = read_existing(&target).await?;
    if previous.is_some() {
        tokio::fs::remove_file(&target)
            .await
            .map_err(|_| "Suppression échouée".to_string())?;
    }
    if let Err(error) = super::storage_index::remove(id).await {
        if restore_previous(target, previous).await.is_err() {
            return Err("Suppression échouée".into());
        }
        return Err(error);
    }
    Ok(())
}

pub async fn rename(id: &str, name: &str) -> Result<ForecastResult, String> {
    validate_analysis_id(id)?;
    let next_name = validate_analysis_name(name)?;
    let mut analysis = load(id).await?;
    analysis.name = next_name;
    save(&mut analysis).await?;
    Ok(analysis)
}

pub async fn list() -> Result<Vec<ForecastAnalysisMeta>, String> {
    super::storage_index::list().await
}

pub async fn comparable_backtests(
    profile: &super::data_quality::DataProfile,
) -> Result<Vec<super::evaluation::types::BacktestIndexSummary>, String> {
    super::storage_backtests::comparable(super::storage_index::entries().await?, profile)
}

fn validate_session_id(session_id: Option<&str>) -> Result<(), String> {
    if let Some(id) = session_id {
        crate::services::agent_local::session_store::validate_session_id(id)
            .map_err(|_| "Identifiant de session invalide".to_string())?;
    }
    Ok(())
}

async fn read_existing(path: &Path) -> Result<Option<Vec<u8>>, String> {
    match super::storage_io::read_bounded(path, MAX_STORED_ANALYSIS_BYTES).await {
        Ok(bytes) => Ok(Some(bytes)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(_) => Err("Erreur de sauvegarde".into()),
    }
}

async fn restore_previous(path: PathBuf, previous: Option<Vec<u8>>) -> Result<(), String> {
    match previous {
        Some(bytes) => crate::services::private_store::atomic_write_async(path, bytes).await,
        None => match tokio::fs::remove_file(path).await {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(_) => Err("Erreur de sauvegarde".into()),
        },
    }
}

async fn cleanup_evicted(ids: Vec<String>) {
    for id in ids {
        if validate_analysis_id(&id).is_err() {
            continue;
        }
        if let Ok(path) = analysis_path_for_write(&id).await {
            let _ = tokio::fs::remove_file(path).await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn optional_session_id_is_validated_before_saving() {
        assert!(validate_session_id(None).is_ok());
        assert!(validate_session_id(Some("550e8400-e29b-41d4-a716-446655440000")).is_ok());
        assert!(validate_session_id(Some("../session")).is_err());
    }

    #[tokio::test]
    async fn rollback_restores_the_previous_analysis_bytes() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("analysis.json");
        tokio::fs::write(&path, b"new").await.unwrap();

        restore_previous(path.clone(), Some(b"old".to_vec()))
            .await
            .unwrap();

        assert_eq!(tokio::fs::read(path).await.unwrap(), b"old");
    }

    #[tokio::test]
    async fn rollback_removes_a_new_analysis_file() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("analysis.json");
        tokio::fs::write(&path, b"new").await.unwrap();

        restore_previous(path.clone(), None).await.unwrap();

        assert!(!path.exists());
    }
}
