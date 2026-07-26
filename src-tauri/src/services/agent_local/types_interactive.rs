use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentInteractiveOption {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub label: String,
    pub description: String,
    #[serde(default)]
    pub recommended: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub preview: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentInteractiveQuestion {
    pub header: String,
    pub question: String,
    pub options: Vec<AgentInteractiveOption>,
    #[serde(default)]
    pub multi_select: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentInteractiveAnswer {
    pub question_index: usize,
    #[serde(default)]
    pub selected_ids: Vec<String>,
    pub selected_labels: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub custom_answer: Option<String>,
}
