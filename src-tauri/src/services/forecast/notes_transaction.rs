use super::notes::ForecastNote;
use super::notes_files::{remove_note, write_note};
use tokio::sync::{Mutex, MutexGuard};

static NOTE_LOCK: Mutex<()> = Mutex::const_new(());
const TRANSACTION_ERROR: &str = "Sauvegarde de note échouée";

pub(super) async fn lock() -> MutexGuard<'static, ()> {
    NOTE_LOCK.lock().await
}

pub(super) async fn rollback_created(note: &ForecastNote, original: String) -> String {
    if remove_note(&note.analysis_id, &note.id).await.is_ok() {
        original
    } else {
        TRANSACTION_ERROR.into()
    }
}

pub(super) async fn rollback_updated(previous: &ForecastNote, original: String) -> String {
    if write_note(previous).await.is_ok() {
        original
    } else {
        TRANSACTION_ERROR.into()
    }
}

pub(super) async fn rollback_deleted(previous: Option<&ForecastNote>, original: String) -> String {
    match previous {
        Some(note) if write_note(note).await.is_err() => TRANSACTION_ERROR.into(),
        _ => original,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::forecast::notes_files::{existing_note_path, load_note, write_note};

    fn note(content: &str) -> ForecastNote {
        ForecastNote {
            id: uuid::Uuid::new_v4().to_string(),
            analysis_id: uuid::Uuid::new_v4().to_string(),
            date: "2026-07-23".into(),
            title: "Test".into(),
            note_type: "context".into(),
            source: "user".into(),
            content: content.into(),
            file_path: String::new(),
            created_at: "2026-07-23T00:00:00Z".into(),
            updated_at: "2026-07-23T00:00:00Z".into(),
        }
    }

    #[tokio::test]
    async fn create_rollback_removes_the_new_note() {
        let item = note("new");
        write_note(&item).await.unwrap();

        let error = rollback_created(&item, "original".into()).await;

        assert_eq!(error, "original");
        assert!(existing_note_path(&item.analysis_id, &item.id)
            .await
            .is_err());
    }

    #[tokio::test]
    async fn update_rollback_restores_the_previous_note() {
        let previous = note("old");
        let mut current = previous.clone();
        current.content = "new".into();
        write_note(&current).await.unwrap();

        let error = rollback_updated(&previous, "original".into()).await;

        assert_eq!(error, "original");
        assert_eq!(
            load_note(&previous.analysis_id, &previous.id)
                .await
                .unwrap()
                .content,
            "old"
        );
    }

    #[tokio::test]
    async fn delete_rollback_restores_the_removed_note() {
        let previous = note("old");

        let error = rollback_deleted(Some(&previous), "original".into()).await;

        assert_eq!(error, "original");
        assert_eq!(
            load_note(&previous.analysis_id, &previous.id)
                .await
                .unwrap()
                .content,
            "old"
        );
    }
}
