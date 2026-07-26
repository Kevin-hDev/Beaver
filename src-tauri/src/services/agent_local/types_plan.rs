use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AgentPlanStatus {
    Draft,
    AwaitingApproval,
    Approved,
    Rejected,
    Cancelled,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AgentPlanWorkflowStatus {
    #[default]
    NeedsContext,
    CollectingQuestions,
    PlanPublished,
    AwaitingApproval,
    Approved,
    Rejected,
    Cancelled,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AgentPlanApprovalDecision {
    Implement,
    ContinuePlanning,
    QuitPlan,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentPlanRun {
    pub id: String,
    pub title: String,
    pub status: AgentPlanStatus,
    pub path: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentPlanPreview {
    pub id: String,
    pub title: String,
    pub content: String,
    pub status: AgentPlanStatus,
}

pub const MAX_PLAN_RUNS: usize = 20;
