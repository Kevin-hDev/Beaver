use super::pricing::ResolvedCost;
use super::request_usage::RequestUsage;
use super::types::{
    LocalSnapshot, RemoteData, UsageBreakdown, UsageOrigin, UsageWorkload, CONNECTION_LIMIT,
    DAY_LIMIT,
};
use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::io::Read;
use std::path::PathBuf;
use std::sync::LazyLock;
use tokio::sync::Mutex;

static STORE_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));
const STORE_SIZE_LIMIT: u64 = 32 * 1024 * 1024;

#[derive(Default, Serialize, Deserialize)]
struct Ledger {
    version: u8,
    connections: BTreeMap<String, ConnectionLedger>,
}

#[derive(Default, Serialize, Deserialize)]
struct ConnectionLedger {
    updated_at: i64,
    all_time: UsageBreakdown,
    days: BTreeMap<String, UsageBreakdown>,
    last_remote: Option<RemoteData>,
    #[serde(default)]
    last_remote_generation: Option<u64>,
}

pub async fn record(
    connection_id: &str,
    origin: UsageOrigin,
    workload: UsageWorkload,
    usage: &RequestUsage,
    cost: ResolvedCost,
) -> Result<(), String> {
    let _guard = STORE_LOCK.lock().await;
    let mut ledger = load();
    let now = Utc::now();
    let connection = connection_mut(&mut ledger, connection_id, now.timestamp());
    let day = connection
        .days
        .entry(now.date_naive().to_string())
        .or_default();
    super::ledger_aggregate::add(day, origin, workload, usage, cost);
    super::ledger_aggregate::add(&mut connection.all_time, origin, workload, usage, cost);
    connection.updated_at = now.timestamp();
    prune_days(&mut connection.days);
    save(&ledger)
}

pub async fn local_snapshot(connection_id: &str) -> LocalSnapshot {
    let _guard = STORE_LOCK.lock().await;
    let ledger = load();
    let Some(connection) = ledger.connections.get(connection_id) else {
        return LocalSnapshot::default();
    };
    let today = Utc::now().date_naive();
    LocalSnapshot {
        today: super::ledger_aggregate::sum_since(&connection.days, today),
        seven_days: super::ledger_aggregate::sum_since(&connection.days, today - Duration::days(6)),
        thirty_days: super::ledger_aggregate::sum_since(
            &connection.days,
            today - Duration::days(29),
        ),
        all_time: connection.all_time.clone(),
    }
}

pub async fn save_remote(
    connection_id: &str,
    generation: u64,
    mut remote: RemoteData,
) -> Result<(), String> {
    let _guard = STORE_LOCK.lock().await;
    let mut ledger = load();
    remote.windows.truncate(super::types::WINDOW_LIMIT);
    remote.balances.truncate(super::types::BALANCE_LIMIT);
    let now = Utc::now().timestamp();
    let connection = connection_mut(&mut ledger, connection_id, now);
    connection.last_remote = Some(remote);
    connection.last_remote_generation = Some(generation);
    connection.updated_at = now;
    save(&ledger)
}

pub async fn recent_remote(connection_id: &str, generation: u64) -> Option<RemoteData> {
    let _guard = STORE_LOCK.lock().await;
    let ledger = load();
    let connection = ledger.connections.get(connection_id)?;
    if connection.last_remote_generation != Some(generation) {
        return None;
    }
    let mut remote = connection.last_remote.clone()?;
    let now = Utc::now().timestamp();
    if !is_recent_timestamp(remote.fetched_at, now) {
        return None;
    }
    remote.stale = true;
    Some(remote)
}

pub async fn clear_remote(connection_id: &str) -> Result<(), String> {
    let _guard = STORE_LOCK.lock().await;
    let mut ledger = load();
    if !clear_remote_in(&mut ledger, connection_id, Utc::now().timestamp()) {
        return Ok(());
    }
    save(&ledger)
}

fn clear_remote_in(ledger: &mut Ledger, connection_id: &str, now: i64) -> bool {
    let Some(connection) = ledger.connections.get_mut(connection_id) else {
        return false;
    };
    if connection.last_remote.take().is_none() {
        return false;
    }
    connection.last_remote_generation = None;
    connection.updated_at = now;
    true
}

fn is_recent_timestamp(fetched_at: i64, now: i64) -> bool {
    fetched_at > 0
        && fetched_at <= now.saturating_add(300)
        && now.saturating_sub(fetched_at) <= 86_400
}

fn connection_mut<'a>(ledger: &'a mut Ledger, id: &str, now: i64) -> &'a mut ConnectionLedger {
    if !ledger.connections.contains_key(id) && ledger.connections.len() >= CONNECTION_LIMIT {
        if let Some(oldest) = ledger
            .connections
            .iter()
            .min_by_key(|(_, value)| value.updated_at)
            .map(|(key, _)| key.clone())
        {
            ledger.connections.remove(&oldest);
        }
    }
    ledger
        .connections
        .entry(id.to_string())
        .or_insert_with(|| ConnectionLedger {
            updated_at: now,
            ..Default::default()
        })
}

fn prune_days(days: &mut BTreeMap<String, UsageBreakdown>) {
    while days.len() > DAY_LIMIT {
        let Some(first) = days.keys().next().cloned() else {
            break;
        };
        days.remove(&first);
    }
}

fn path() -> PathBuf {
    crate::services::paths::data_dir().join("provider-usage.json")
}

fn load() -> Ledger {
    let mut ledger = read_bounded()
        .map(|bytes| decode(&bytes))
        .unwrap_or_default();
    ledger.version = 1;
    while ledger.connections.len() > CONNECTION_LIMIT {
        let Some(first) = ledger.connections.keys().next().cloned() else {
            break;
        };
        ledger.connections.remove(&first);
    }
    for connection in ledger.connections.values_mut() {
        prune_days(&mut connection.days);
        if let Some(remote) = &mut connection.last_remote {
            remote.windows.truncate(super::types::WINDOW_LIMIT);
            remote.balances.truncate(super::types::BALANCE_LIMIT);
            for window in &mut remote.windows {
                window.resets_at = window
                    .resets_at
                    .and_then(super::types::valid_reset_timestamp);
            }
        }
    }
    ledger
}

fn read_bounded() -> Option<Vec<u8>> {
    let file = std::fs::File::open(path()).ok()?;
    let mut bytes = Vec::new();
    file.take(STORE_SIZE_LIMIT.saturating_add(1))
        .read_to_end(&mut bytes)
        .ok()?;
    (bytes.len() as u64 <= STORE_SIZE_LIMIT).then_some(bytes)
}

fn decode(bytes: &[u8]) -> Ledger {
    serde_json::from_slice(bytes).unwrap_or_default()
}

fn save(ledger: &Ledger) -> Result<(), String> {
    let bytes =
        serde_json::to_vec(ledger).map_err(|_| "Stockage usage indisponible".to_string())?;
    crate::services::private_store::atomic_write(&path(), &bytes)
}

#[cfg(test)]
#[path = "ledger_tests.rs"]
mod tests;
