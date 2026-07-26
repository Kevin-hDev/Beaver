use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForecastNote {
    pub id: String,
    pub analysis_id: String,
    pub date: String,
    pub title: String,
    pub note_type: String,
    pub source: String,
    pub content: String,
    pub file_path: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct ForecastNoteCreateRequest {
    pub analysis_id: String,
    pub date: String,
    pub title: String,
    pub note_type: String,
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct ForecastNoteUpdateRequest {
    pub analysis_id: String,
    pub note_id: String,
    pub date: String,
    pub title: String,
    pub note_type: String,
    pub content: String,
}

pub(crate) struct NoteMutation<T> {
    pub value: T,
    pub revision: u32,
    pub session_id: Option<String>,
}

pub(crate) struct NoteListResult {
    pub notes: Vec<ForecastNote>,
    pub revision: Option<u32>,
    pub session_id: Option<String>,
}
