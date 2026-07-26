use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentTodoStatus {
    Pending,
    InProgress,
    Completed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentTodoItem {
    pub content: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active_form: Option<String>,
    pub status: AgentTodoStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentTodoRunStatus {
    Active,
    Paused,
    Completed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentTodoRun {
    pub id: String,
    pub title: String,
    pub status: AgentTodoRunStatus,
    pub todos: Vec<AgentTodoItem>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub paused_reason: Option<String>,
}

pub fn all_completed(todos: &[AgentTodoItem]) -> bool {
    !todos.is_empty()
        && todos
            .iter()
            .all(|todo| todo.status == AgentTodoStatus::Completed)
}
