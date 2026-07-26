use serde::{Deserialize, Serialize};

pub const CONNECTION_LIMIT: usize = 32;
pub const DAY_LIMIT: usize = 400;
pub const WINDOW_LIMIT: usize = 8;
pub const BALANCE_LIMIT: usize = 4;
const MIN_RESET_TIMESTAMP: i64 = 946_684_800;
const MAX_RESET_HORIZON_SECONDS: i64 = 366 * 24 * 60 * 60;

const CONNECTIONS: &[&str] = &[
    "groq",
    "google",
    "mistral",
    "cerebras",
    "openrouter",
    "openai",
    "deepseek",
    "xai",
    "moonshot",
    "zai",
    "codex-oauth",
    "xai-oauth",
    "moonshot-oauth",
];
pub(super) const CONNECTION_COUNT: usize = CONNECTIONS.len();

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UsageOrigin {
    #[default]
    ManualChat,
    ExternalChannel,
    Automation,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UsageWorkload {
    #[default]
    Primary,
    Subagent,
    Compression,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct TokenTotals {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cached_input_tokens: u64,
    pub reasoning_output_tokens: u64,
    pub total_tokens: u64,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct UsageAggregate {
    pub tokens: TokenTotals,
    pub request_count: u64,
    pub usage_request_count: u64,
    pub cost_usd_micros: u64,
    pub priced_request_count: u64,
    pub exact_cost_request_count: u64,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct OriginBreakdown {
    pub manual_chat: UsageAggregate,
    pub external_channel: UsageAggregate,
    pub automation: UsageAggregate,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkloadBreakdown {
    pub primary: UsageAggregate,
    pub subagent: UsageAggregate,
    pub compression: UsageAggregate,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UsageBreakdown {
    pub totals: UsageAggregate,
    pub origins: OriginBreakdown,
    pub workloads: WorkloadBreakdown,
}

#[derive(Debug, Clone, Default)]
pub struct LocalSnapshot {
    pub today: UsageBreakdown,
    pub seven_days: UsageBreakdown,
    pub thirty_days: UsageBreakdown,
    pub all_time: UsageBreakdown,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProviderWindow {
    pub label_code: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub group_code: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub group_name: Option<String>,
    pub used: Option<f64>,
    pub limit: Option<f64>,
    pub remaining: Option<f64>,
    pub used_percent: Option<f64>,
    pub resets_at: Option<i64>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProviderBalance {
    pub label_code: String,
    pub amount: String,
    pub currency: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RemoteData {
    pub windows: Vec<ProviderWindow>,
    pub balances: Vec<ProviderBalance>,
    pub notice_code: Option<String>,
    pub fetched_at: i64,
    pub stale: bool,
}

pub fn validate_connection_id(value: &str) -> Result<(), String> {
    CONNECTIONS
        .contains(&value)
        .then_some(())
        .ok_or_else(|| "Fournisseur invalide".to_string())
}

pub(super) fn connection_index(value: &str) -> Option<usize> {
    CONNECTIONS.iter().position(|candidate| *candidate == value)
}

pub(super) fn valid_reset_timestamp(value: i64) -> Option<i64> {
    let maximum = chrono::Utc::now()
        .timestamp()
        .saturating_add(MAX_RESET_HORIZON_SECONDS);
    (value >= MIN_RESET_TIMESTAMP && value <= maximum).then_some(value)
}

pub async fn context_for_session(
    session_id: &str,
    requested_workload: UsageWorkload,
) -> (UsageOrigin, UsageWorkload) {
    let origin = origin_for_session(session_id)
        .await
        .unwrap_or(UsageOrigin::ManualChat);
    let Ok(session) = crate::services::agent_local::session_store::get(session_id).await else {
        return (origin, requested_workload);
    };
    let workload = if requested_workload == UsageWorkload::Primary
        && (session.parent_session_id.is_some() || session.subagent_type.is_some())
    {
        UsageWorkload::Subagent
    } else {
        requested_workload
    };
    (origin, workload)
}

pub(crate) async fn origin_for_session(session_id: &str) -> Option<UsageOrigin> {
    let mut current_id = session_id.to_string();
    for _ in 0..=8 {
        let session = crate::services::agent_local::session_store::get(&current_id)
            .await
            .ok()?;
        if session.is_heartbeat {
            return Some(UsageOrigin::Automation);
        }
        if session.is_gateway {
            return Some(UsageOrigin::ExternalChannel);
        }
        let Some(parent_id) = session.parent_session_id else {
            return Some(UsageOrigin::ManualChat);
        };
        current_id = parent_id;
    }
    None
}

#[cfg(test)]
mod tests {
    use super::valid_reset_timestamp;

    #[test]
    fn reset_timestamps_must_be_plausible() {
        let soon = chrono::Utc::now().timestamp().saturating_add(3_600);
        assert_eq!(valid_reset_timestamp(soon), Some(soon));
        assert_eq!(valid_reset_timestamp(i64::MAX), None);
        assert_eq!(valid_reset_timestamp(-1), None);
    }
}
