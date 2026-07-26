use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Annotation {
    pub id: String,
    pub date: String,
    pub text: String,
    pub source: AnnotationSource,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note_title: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note_content: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note_created_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note_updated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AnnotationSource {
    User,
    Llm,
}
