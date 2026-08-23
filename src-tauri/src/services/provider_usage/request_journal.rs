use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::LazyLock;
use tokio::sync::Mutex;

use super::{RequestUsage, UsageApiFormat};

pub(super) const REQUEST_LIMIT: usize = 1_000;
const SESSION_REQUEST_LIMIT: usize = 200;
const SNAPSHOT_LIMIT: usize = 50;
static STORE_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));
static STORE_UNAVAILABLE: AtomicBool = AtomicBool::new(false);

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RequestMetricStatus {
    #[default]
    Failed,
    Completed,
    Interrupted,
    Cancelled,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ServiceTierServed {
    Fast,
    Default,
    #[default]
    #[serde(other)]
    Unknown,
}

pub(super) fn served_tier(value: &str) -> ServiceTierServed {
    match value {
        "fast" | "priority" => ServiceTierServed::Fast,
        "default" => ServiceTierServed::Default,
        _ => ServiceTierServed::Unknown,
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct RequestTiming {
    pub headers_ms: Option<u64>,
    pub first_event_ms: Option<u64>,
    pub first_useful_ms: Option<u64>,
    pub total_ms: u64,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ProviderRequestMetric {
    pub started_at_ms: i64,
    pub connection_id: String,
    pub canonical_provider_id: String,
    pub api_format: UsageApiFormat,
    pub model: String,
    pub routed_provider: Option<String>,
    pub routed_model: Option<String>,
    pub session_id: Option<String>,
    pub request_id: String,
    pub turn: Option<u32>,
    pub attempt: u32,
    pub workload: String,
    pub origin: String,
    pub status: RequestMetricStatus,
    pub fast_requested: bool,
    pub service_tier_served: ServiceTierServed,
    pub timing: RequestTiming,
    pub usage: Option<RequestUsage>,
    pub usage_complete: bool,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct RequestMetricsSnapshot {
    pub availability: &'static str,
    pub recent: Vec<ProviderRequestMetric>,
    pub sessions: Vec<super::request_journal_summary::RequestSessionSummary>,
}

pub async fn record(metric: ProviderRequestMetric) -> Result<(), String> {
    if !metric.is_valid() {
        STORE_UNAVAILABLE.store(true, Ordering::Release);
        return Err("Mesure provider invalide".into());
    }
    let _guard = STORE_LOCK.lock().await;
    let mut store = match super::request_journal_store::load() {
        Ok(store) => store,
        Err(error) => {
            STORE_UNAVAILABLE.store(true, Ordering::Release);
            return Err(error);
        }
    };
    store.entries.retain(ProviderRequestMetric::is_valid);
    store.entries.push(metric);
    prune(&mut store.entries);
    let result = super::request_journal_store::save(&store);
    STORE_UNAVAILABLE.store(result.is_err(), Ordering::Release);
    result
}

pub async fn snapshot(connection_id: &str) -> RequestMetricsSnapshot {
    if STORE_UNAVAILABLE.load(Ordering::Acquire) {
        return unavailable_snapshot();
    }
    let _guard = STORE_LOCK.lock().await;
    match super::request_journal_store::load() {
        Ok(mut store) => {
            store.entries.retain(ProviderRequestMetric::is_valid);
            prune(&mut store.entries);
            let sessions =
                super::request_journal_summary::session_summaries(&store.entries, connection_id);
            let mut recent: Vec<_> = store
                .entries
                .into_iter()
                .rev()
                .filter(|metric| metric.connection_id == connection_id)
                .take(SNAPSHOT_LIMIT)
                .collect();
            recent.reverse();
            RequestMetricsSnapshot {
                availability: if recent.is_empty() {
                    "empty"
                } else {
                    "complete"
                },
                recent,
                sessions,
            }
        }
        Err(_) => unavailable_snapshot(),
    }
}

fn unavailable_snapshot() -> RequestMetricsSnapshot {
    RequestMetricsSnapshot {
        availability: "unavailable",
        recent: Vec::new(),
        sessions: Vec::new(),
    }
}

pub(super) fn prune(entries: &mut Vec<ProviderRequestMetric>) {
    while entries.len() > REQUEST_LIMIT {
        entries.remove(0);
    }
    let sessions: Vec<String> = entries
        .iter()
        .filter_map(|m| m.session_id.clone())
        .collect();
    for session_id in sessions {
        while entries
            .iter()
            .filter(|metric| metric.session_id.as_deref() == Some(&session_id))
            .count()
            > SESSION_REQUEST_LIMIT
        {
            let Some(index) = entries
                .iter()
                .position(|metric| metric.session_id.as_deref() == Some(&session_id))
            else {
                break;
            };
            entries.remove(index);
        }
    }
}
