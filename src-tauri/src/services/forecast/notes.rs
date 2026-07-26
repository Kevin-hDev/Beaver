use super::notes_files::{
    existing_note_path, load_note, load_note_if_exists, read_notes, remove_note,
    sync_annotation_files, write_note,
};
use super::notes_types::{NoteListResult, NoteMutation};
use super::{notes_annotations, notes_transaction, notes_validation, storage};
use crate::services::forecast::types::{ForecastResult, MAX_ANNOTATIONS};
use std::ffi::OsStr;
use std::process::Command;

pub use super::notes_types::{ForecastNote, ForecastNoteCreateRequest, ForecastNoteUpdateRequest};

pub(crate) async fn list(analysis_id: &str) -> Result<NoteListResult, String> {
    notes_validation::id(analysis_id, "Identifiant d'analyse invalide")?;
    let _guard = notes_transaction::lock().await;
    let mut analysis = storage::load(analysis_id).await?;
    sync_annotation_files(&analysis).await?;
    let notes = read_notes(analysis_id).await?;
    let mut changed = false;
    for note in &notes {
        changed |= notes_annotations::upsert(&mut analysis, note)?;
    }
    if changed {
        storage::save(&mut analysis).await?;
    }
    Ok(NoteListResult {
        notes,
        revision: changed.then_some(analysis.revision),
        session_id: analysis.session_id,
    })
}

pub(crate) async fn create(
    request: ForecastNoteCreateRequest,
) -> Result<NoteMutation<ForecastNote>, String> {
    notes_validation::id(&request.analysis_id, "Identifiant d'analyse invalide")?;
    let date = notes_validation::field(&request.date)?;
    let title = notes_validation::title(&request.title)?;
    let note_type = notes_validation::note_type(&request.note_type)?;
    let content = notes_validation::content(&request.content)?;
    let _guard = notes_transaction::lock().await;
    let mut analysis = storage::load(&request.analysis_id).await?;
    if analysis.annotations.len() >= MAX_ANNOTATIONS {
        return Err("Limite de notes atteinte".into());
    }
    let now = chrono::Utc::now().to_rfc3339();
    let note = ForecastNote {
        id: uuid::Uuid::new_v4().to_string(),
        analysis_id: request.analysis_id,
        date,
        title,
        note_type,
        source: "user".into(),
        content,
        file_path: String::new(),
        created_at: now.clone(),
        updated_at: now,
    };
    notes_annotations::upsert(&mut analysis, &note)?;
    write_note(&note).await?;
    if let Err(error) = storage::save(&mut analysis).await {
        return Err(notes_transaction::rollback_created(&note, error).await);
    }
    let saved = load_note(&note.analysis_id, &note.id).await?;
    Ok(mutation(saved, &analysis))
}

pub(crate) async fn update(
    request: ForecastNoteUpdateRequest,
) -> Result<NoteMutation<ForecastNote>, String> {
    notes_validation::id(&request.analysis_id, "Identifiant d'analyse invalide")?;
    notes_validation::id(&request.note_id, "Identifiant de note invalide")?;
    let date = notes_validation::field(&request.date)?;
    let title = notes_validation::title(&request.title)?;
    let note_type = notes_validation::note_type(&request.note_type)?;
    let content = notes_validation::content(&request.content)?;
    let _guard = notes_transaction::lock().await;
    let mut current = load_note(&request.analysis_id, &request.note_id).await?;
    let previous = current.clone();
    current.date = date;
    current.title = title;
    current.note_type = note_type;
    current.content = content;
    current.updated_at = chrono::Utc::now().to_rfc3339();
    let mut analysis = storage::load(&request.analysis_id).await?;
    notes_annotations::upsert(&mut analysis, &current)?;
    write_note(&current).await?;
    if let Err(error) = storage::save(&mut analysis).await {
        return Err(notes_transaction::rollback_updated(&previous, error).await);
    }
    let saved = load_note(&request.analysis_id, &request.note_id).await?;
    Ok(mutation(saved, &analysis))
}

pub(crate) async fn delete(analysis_id: &str, note_id: &str) -> Result<NoteMutation<()>, String> {
    notes_validation::id(analysis_id, "Identifiant d'analyse invalide")?;
    notes_validation::id(note_id, "Identifiant de note invalide")?;
    let _guard = notes_transaction::lock().await;
    let mut analysis = storage::load(analysis_id).await?;
    let previous = load_note_if_exists(analysis_id, note_id).await?;
    if let Some(note) = previous.as_ref() {
        if notes_annotations::upsert(&mut analysis, note)? {
            storage::save(&mut analysis).await?;
        }
    }
    remove_note(analysis_id, note_id).await?;
    analysis
        .annotations
        .retain(|annotation| annotation.id != note_id);
    if let Err(error) = storage::save(&mut analysis).await {
        return Err(notes_transaction::rollback_deleted(previous.as_ref(), error).await);
    }
    Ok(mutation((), &analysis))
}

pub async fn open(analysis_id: &str, note_id: &str) -> Result<(), String> {
    notes_validation::id(analysis_id, "Identifiant d'analyse invalide")?;
    notes_validation::id(note_id, "Identifiant de note invalide")?;
    let path = existing_note_path(analysis_id, note_id).await?;
    #[cfg(target_os = "macos")]
    return spawn_cmd("open", &[path.as_os_str()]);
    #[cfg(target_os = "linux")]
    return spawn_cmd("xdg-open", &[path.as_os_str()]);
    #[cfg(target_os = "windows")]
    return spawn_cmd("explorer.exe", &[path.as_os_str()]);
}

fn mutation<T>(value: T, analysis: &ForecastResult) -> NoteMutation<T> {
    NoteMutation {
        value,
        revision: analysis.revision,
        session_id: analysis.session_id.clone(),
    }
}

fn spawn_cmd(command: &str, args: &[&OsStr]) -> Result<(), String> {
    Command::new(command)
        .args(args)
        .spawn()
        .map(|_| ())
        .map_err(|_| "Impossible d'ouvrir la note".to_string())
}
