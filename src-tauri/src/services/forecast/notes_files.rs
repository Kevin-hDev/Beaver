use super::notes_annotations;
use super::notes_format;
use super::notes_paths;
use super::notes_types::ForecastNote;
use super::notes_validation;
use crate::services::forecast::types::{ForecastResult, MAX_ANNOTATIONS};
use crate::services::private_store;
use std::path::PathBuf;

const MAX_NOTE_DIRECTORY_ENTRIES: usize = 512;

pub(crate) async fn read_notes(analysis_id: &str) -> Result<Vec<ForecastNote>, String> {
    let Some(directory) = notes_paths::directory_if_exists(analysis_id).await? else {
        return Ok(Vec::new());
    };
    let mut entries = tokio::fs::read_dir(&directory)
        .await
        .map_err(|_| "Impossible de lire les notes".to_string())?;
    let mut notes = Vec::new();
    let mut scanned = 0;
    while let Some(entry) = entries
        .next_entry()
        .await
        .map_err(|_| "Impossible de lire les notes".to_string())?
    {
        scanned += 1;
        if scanned > MAX_NOTE_DIRECTORY_ENTRIES {
            return Err("Trop de fichiers de notes".into());
        }
        let path = entry.path();
        if path.extension().is_none_or(|extension| extension != "md") {
            continue;
        }
        let Some(note_id) = path.file_stem().and_then(|value| value.to_str()) else {
            continue;
        };
        if notes_validation::id(note_id, "Note invalide").is_err() {
            continue;
        }
        let path = notes_paths::verify_directory_entry(&directory, &path).await?;
        notes.push(read_note_file(path, analysis_id, note_id).await?);
        if notes.len() >= MAX_ANNOTATIONS {
            break;
        }
    }
    notes.sort_by(|a, b| a.date.cmp(&b.date).then(a.created_at.cmp(&b.created_at)));
    Ok(notes)
}

pub(crate) async fn load_note(analysis_id: &str, note_id: &str) -> Result<ForecastNote, String> {
    load_note_if_exists(analysis_id, note_id)
        .await?
        .ok_or_else(|| "Note introuvable".to_string())
}

pub(crate) async fn load_note_if_exists(
    analysis_id: &str,
    note_id: &str,
) -> Result<Option<ForecastNote>, String> {
    let Some(path) = notes_paths::file_if_exists(analysis_id, note_id).await? else {
        return Ok(None);
    };
    read_note_file(path, analysis_id, note_id).await.map(Some)
}

pub(crate) async fn write_note(note: &ForecastNote) -> Result<(), String> {
    validate_note(note)?;
    let path = notes_paths::file_for_write(&note.analysis_id, &note.id).await?;
    let bytes = notes_format::serialize(note).into_bytes();
    if bytes.len() > notes_validation::MAX_NOTE_FILE_BYTES {
        return Err("Note trop volumineuse".into());
    }
    private_store::atomic_write_async(path, bytes)
        .await
        .map_err(|_| "Sauvegarde de note échouée".to_string())
}

pub(crate) async fn remove_note(analysis_id: &str, note_id: &str) -> Result<(), String> {
    let Some(path) = notes_paths::file_if_exists(analysis_id, note_id).await? else {
        return Ok(());
    };
    tokio::fs::remove_file(path)
        .await
        .map_err(|_| "Suppression échouée".to_string())
}

pub(crate) async fn existing_note_path(
    analysis_id: &str,
    note_id: &str,
) -> Result<PathBuf, String> {
    notes_paths::file_if_exists(analysis_id, note_id)
        .await?
        .ok_or_else(|| "Note introuvable".to_string())
}

pub(crate) async fn sync_annotation_files(analysis: &ForecastResult) -> Result<(), String> {
    if analysis.annotations.is_empty() {
        return Ok(());
    }
    notes_paths::directory_for_write(&analysis.id).await?;
    for annotation in &analysis.annotations {
        if notes_validation::id(&annotation.id, "Note invalide").is_err() {
            continue;
        }
        if notes_paths::file_if_exists(&analysis.id, &annotation.id)
            .await?
            .is_some()
        {
            continue;
        }
        write_note(&notes_annotations::note_from_annotation(
            analysis, annotation,
        )?)
        .await?;
    }
    Ok(())
}

async fn read_note_file(
    path: PathBuf,
    analysis_id: &str,
    note_id: &str,
) -> Result<ForecastNote, String> {
    let metadata = tokio::fs::metadata(&path)
        .await
        .map_err(|_| "Lecture de note échouée".to_string())?;
    if !metadata.is_file() || metadata.len() > notes_validation::MAX_NOTE_FILE_BYTES as u64 {
        return Err("Note invalide".into());
    }
    let raw = tokio::fs::read_to_string(&path)
        .await
        .map_err(|_| "Lecture de note échouée".to_string())?;
    let mut note = notes_format::parse(&raw, analysis_id, note_id)?;
    note.file_path = path.to_string_lossy().to_string();
    Ok(note)
}

fn validate_note(note: &ForecastNote) -> Result<(), String> {
    notes_validation::id(&note.analysis_id, "Identifiant d'analyse invalide")?;
    notes_validation::id(&note.id, "Identifiant de note invalide")?;
    notes_validation::field(&note.date)?;
    notes_validation::title(&note.title)?;
    notes_validation::note_type(&note.note_type)?;
    notes_validation::source(&note.source)?;
    notes_validation::content(&note.content)?;
    notes_validation::metadata(&note.created_at)?;
    notes_validation::metadata(&note.updated_at)?;
    Ok(())
}
