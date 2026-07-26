use super::notes;
use super::notes_files::write_note;
use super::notes_paths;
use super::notes_types::{ForecastNote, ForecastNoteCreateRequest, ForecastNoteUpdateRequest};
use super::storage;
use super::types::ForecastResult;

#[tokio::test]
async fn orphan_note_is_reconciled_into_the_analysis() {
    let mut analysis = analysis();
    storage::save(&mut analysis).await.unwrap();
    let note = note(&analysis.id);
    write_note(&note).await.unwrap();

    let result = notes::list(&analysis.id).await.unwrap();
    let stored = storage::load(&analysis.id).await.unwrap();

    assert_eq!(result.notes.len(), 1);
    assert_eq!(result.revision, Some(stored.revision));
    assert_eq!(stored.annotations.len(), 1);
    assert_eq!(
        stored.annotations[0].note_content.as_deref(),
        Some("Contenu important")
    );
}

#[tokio::test]
async fn missing_note_file_is_rebuilt_without_losing_metadata() {
    let mut analysis = analysis();
    let item = note(&analysis.id);
    super::notes_annotations::upsert(&mut analysis, &item).unwrap();
    storage::save(&mut analysis).await.unwrap();

    let result = notes::list(&analysis.id).await.unwrap();

    assert_eq!(result.revision, None);
    assert_eq!(result.notes[0].title, "Titre distinct");
    assert_eq!(result.notes[0].note_type, "decision");
    assert_eq!(result.notes[0].content, "Contenu important");
}

#[tokio::test]
async fn listing_an_analysis_without_notes_creates_no_empty_directory() {
    let mut analysis = analysis();
    storage::save(&mut analysis).await.unwrap();

    let result = notes::list(&analysis.id).await.unwrap();

    assert!(result.notes.is_empty());
    assert!(notes_paths::directory_if_exists(&analysis.id)
        .await
        .unwrap()
        .is_none());
}

#[tokio::test]
async fn corrupted_note_fails_closed_without_panicking() {
    let mut analysis = analysis();
    storage::save(&mut analysis).await.unwrap();
    let note_id = uuid::Uuid::new_v4().to_string();
    let path = notes_paths::file_for_write(&analysis.id, &note_id)
        .await
        .unwrap();
    crate::services::private_store::atomic_write_async(path, b"\n---\n".to_vec())
        .await
        .unwrap();

    assert!(notes::list(&analysis.id).await.is_err());
    assert!(super::export::export_analysis(&analysis.id, "clipboard")
        .await
        .is_err());
}

#[cfg(unix)]
#[tokio::test]
async fn symlinked_analysis_directory_is_rejected() {
    use std::os::unix::fs::symlink;

    let analysis_id = uuid::Uuid::new_v4().to_string();
    let root = notes_paths::root_for_write().await.unwrap();
    let outside = tempfile::tempdir().unwrap();
    let link = root.join(&analysis_id);
    symlink(outside.path(), &link).unwrap();

    assert!(notes_paths::directory_for_write(&analysis_id)
        .await
        .is_err());
    std::fs::remove_file(link).unwrap();
}

#[cfg(unix)]
#[tokio::test]
async fn note_files_are_private() {
    use std::os::unix::fs::PermissionsExt;

    let analysis_id = uuid::Uuid::new_v4().to_string();
    let item = note(&analysis_id);
    write_note(&item).await.unwrap();
    let path = notes_paths::file_if_exists(&analysis_id, &item.id)
        .await
        .unwrap()
        .unwrap();
    let mode = std::fs::metadata(path).unwrap().permissions().mode();

    assert_eq!(mode & 0o077, 0);
}

#[tokio::test]
async fn note_mutations_return_the_persisted_revision() {
    let mut stored_analysis = analysis();
    storage::save(&mut stored_analysis).await.unwrap();
    let created = notes::create(ForecastNoteCreateRequest {
        analysis_id: stored_analysis.id.clone(),
        date: "2026-07-23".into(),
        title: "Créée".into(),
        note_type: "context".into(),
        content: "Version initiale".into(),
    })
    .await
    .unwrap();
    assert_eq!(
        created.revision,
        storage::load(&stored_analysis.id).await.unwrap().revision
    );

    let updated = notes::update(ForecastNoteUpdateRequest {
        analysis_id: stored_analysis.id.clone(),
        note_id: created.value.id.clone(),
        date: "2026-07-24".into(),
        title: "Mise à jour".into(),
        note_type: "decision".into(),
        content: "Version finale".into(),
    })
    .await
    .unwrap();
    assert_eq!(
        updated.revision,
        storage::load(&stored_analysis.id).await.unwrap().revision
    );

    let deleted = notes::delete(&stored_analysis.id, &created.value.id)
        .await
        .unwrap();
    assert_eq!(
        deleted.revision,
        storage::load(&stored_analysis.id).await.unwrap().revision
    );
}

fn analysis() -> ForecastResult {
    let id = uuid::Uuid::new_v4().to_string();
    serde_json::from_value(serde_json::json!({
        "id": id,
        "name": "Analyse notes",
        "target_column": "sales",
        "created_at": "2026-07-23T00:00:00Z",
        "session_id": null,
        "model": "naive",
        "provider": "local",
        "horizon": 1,
        "frequency": "D",
        "input_summary": {"points": 2, "start": "2026-07-21", "end": "2026-07-22"},
        "predictions": [{"date": "2026-07-23", "value": 1.0}],
        "quantiles": {"q10": [0.5], "q50": [1.0], "q90": [1.5]}
    }))
    .unwrap()
}

fn note(analysis_id: &str) -> ForecastNote {
    ForecastNote {
        id: uuid::Uuid::new_v4().to_string(),
        analysis_id: analysis_id.into(),
        date: "2026-07-23".into(),
        title: "Titre distinct".into(),
        note_type: "decision".into(),
        source: "user".into(),
        content: "Contenu important".into(),
        file_path: String::new(),
        created_at: "2026-07-23T00:00:00Z".into(),
        updated_at: "2026-07-23T00:00:00Z".into(),
    }
}
