use super::types::{
    LocalSnapshot, ProviderBalance, ProviderWindow, RemoteData, UsageAggregate, UsageBreakdown,
};
use serde::Serialize;

#[derive(Debug, Clone, Default, Serialize)]
pub struct UsagePeriod {
    pub period: &'static str,
    pub totals: UsageAggregate,
    pub origins: super::types::OriginBreakdown,
    pub workloads: super::types::WorkloadBreakdown,
    pub cost_quality: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProviderUsageSnapshot {
    pub connection_id: String,
    pub canonical_provider_id: String,
    pub auth_source: &'static str,
    pub availability: &'static str,
    pub windows: Vec<ProviderWindow>,
    pub balances: Vec<ProviderBalance>,
    pub local_periods: Vec<UsagePeriod>,
    pub notice_code: Option<String>,
    pub refreshed_at: i64,
    pub stale: bool,
}

pub fn build_snapshot(
    connection_id: &str,
    local: LocalSnapshot,
    remote: RemoteData,
) -> ProviderUsageSnapshot {
    let has_local_usage = local.all_time.totals.request_count > 0;
    let availability = if remote.notice_code.as_deref() == Some("usage_fetch_failed")
        && !has_local_usage
        && remote.windows.is_empty()
        && remote.balances.is_empty()
    {
        "unavailable"
    } else if remote.windows.is_empty() && remote.balances.is_empty() {
        "partial"
    } else {
        "complete"
    };
    ProviderUsageSnapshot {
        connection_id: connection_id.to_string(),
        canonical_provider_id: canonical_id(connection_id).to_string(),
        auth_source: if connection_id.ends_with("-oauth") {
            "oauth"
        } else {
            "api"
        },
        availability,
        windows: remote.windows,
        balances: remote.balances,
        local_periods: periods(local),
        notice_code: remote.notice_code,
        refreshed_at: if remote.fetched_at > 0 {
            remote.fetched_at
        } else {
            chrono::Utc::now().timestamp()
        },
        stale: remote.stale,
    }
}

fn periods(local: LocalSnapshot) -> Vec<UsagePeriod> {
    vec![
        period("today", local.today),
        period("seven_days", local.seven_days),
        period("thirty_days", local.thirty_days),
        period("all_time", local.all_time),
    ]
}

fn period(period: &'static str, breakdown: UsageBreakdown) -> UsagePeriod {
    let totals = breakdown.totals;
    let cost_quality = cost_quality(&totals);
    UsagePeriod {
        period,
        totals,
        origins: breakdown.origins,
        workloads: breakdown.workloads,
        cost_quality,
    }
}

fn cost_quality(total: &UsageAggregate) -> &'static str {
    if total.priced_request_count == 0 {
        "unavailable"
    } else if total.priced_request_count < total.request_count {
        "partial"
    } else if total.exact_cost_request_count == total.request_count {
        "exact"
    } else {
        "estimated"
    }
}

fn canonical_id(connection_id: &str) -> &str {
    match connection_id {
        "codex-oauth" => "openai",
        "xai-oauth" => "xai",
        "moonshot-oauth" => "moonshot",
        other => other,
    }
}
