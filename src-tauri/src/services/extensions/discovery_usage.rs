use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::io::Read;
use std::path::PathBuf;
use std::sync::{LazyLock, Mutex};

use super::discovery_limits::DISCOVERY_STORE_MAX_BYTES;
use super::types::MAX_EXTENSIONS;

const WEEK_SECONDS: f64 = 604_800.0;
const WEEKLY_DECAY: f64 = 0.8;
const SCORE_FLOOR: f64 = 0.01;
static STORE_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

#[derive(Clone, Default, Deserialize, Serialize, PartialEq)]
struct UsageLedger {
    #[serde(default)]
    entries: BTreeMap<String, UsageEntry>,
}

#[derive(Clone, Deserialize, Serialize, PartialEq)]
struct UsageEntry {
    score: f64,
    updated_at: i64,
}

pub fn scores() -> Result<BTreeMap<String, f64>, String> {
    let _guard = STORE_LOCK
        .lock()
        .map_err(|_| "Compteur d'usage indisponible.".to_string())?;
    let mut ledger = load();
    let previous = ledger.clone();
    let now = Utc::now().timestamp();
    prune(&mut ledger, now);
    if ledger != previous {
        save(&ledger)?;
    }
    Ok(ledger
        .entries
        .into_iter()
        .map(|(id, entry)| (id, decayed(&entry, now)))
        .collect())
}

pub fn record_tool(tool_name: &str) -> Result<(), String> {
    let Some(plugin_id) = super::registry_index::plugin_id_for_tool(tool_name) else {
        return Ok(());
    };
    let _guard = STORE_LOCK
        .lock()
        .map_err(|_| "Compteur d'usage indisponible.".to_string())?;
    let mut ledger = load();
    let now = Utc::now().timestamp();
    prune(&mut ledger, now);
    if !ledger.entries.contains_key(&plugin_id) && ledger.entries.len() >= MAX_EXTENSIONS {
        evict_lowest(&mut ledger, now);
    }
    let entry = ledger.entries.entry(plugin_id).or_insert(UsageEntry {
        score: 0.0,
        updated_at: now,
    });
    entry.score = decayed(entry, now) + 1.0;
    entry.updated_at = now;
    save(&ledger)
}

fn decayed(entry: &UsageEntry, now: i64) -> f64 {
    let elapsed = now.saturating_sub(entry.updated_at).max(0) as f64;
    entry.score * WEEKLY_DECAY.powf(elapsed / WEEK_SECONDS)
}

fn prune(ledger: &mut UsageLedger, now: i64) {
    ledger.entries.retain(|id, entry| {
        super::validation::identifier(id).is_ok() && decayed(entry, now) >= SCORE_FLOOR
    });
    while ledger.entries.len() > MAX_EXTENSIONS {
        evict_lowest(ledger, now);
    }
}

fn evict_lowest(ledger: &mut UsageLedger, now: i64) {
    let candidate = ledger
        .entries
        .iter()
        .min_by(|(_, left), (_, right)| {
            decayed(left, now)
                .total_cmp(&decayed(right, now))
                .then_with(|| left.updated_at.cmp(&right.updated_at))
        })
        .map(|(id, _)| id.clone());
    if let Some(id) = candidate {
        ledger.entries.remove(&id);
    }
}

fn path() -> PathBuf {
    crate::services::paths::data_dir().join("extension-tool-usage.json")
}

fn load() -> UsageLedger {
    let Some(bytes) = read_bounded() else {
        return UsageLedger::default();
    };
    serde_json::from_slice(&bytes).unwrap_or_default()
}

fn read_bounded() -> Option<Vec<u8>> {
    let file = std::fs::File::open(path()).ok()?;
    let mut bytes = Vec::new();
    file.take(DISCOVERY_STORE_MAX_BYTES.saturating_add(1))
        .read_to_end(&mut bytes)
        .ok()?;
    (bytes.len() as u64 <= DISCOVERY_STORE_MAX_BYTES).then_some(bytes)
}

fn save(ledger: &UsageLedger) -> Result<(), String> {
    let bytes =
        serde_json::to_vec(ledger).map_err(|_| "Compteur d'usage indisponible.".to_string())?;
    if bytes.len() as u64 > DISCOVERY_STORE_MAX_BYTES {
        return Err("Compteur d'usage invalide.".to_string());
    }
    crate::services::private_store::atomic_write(&path(), &bytes)
        .map_err(|_| "Compteur d'usage indisponible.".to_string())
}

#[cfg(test)]
#[path = "discovery_usage_tests.rs"]
mod tests;
