use serde::{Deserialize, Deserializer, Serialize};
use std::io::ErrorKind;
use std::path::PathBuf;
use tokio::io::AsyncReadExt;

use super::tool_metrics::{ToolMetricEntry, MAX_TRACKED_TOOLS};

const STORE_VERSION: u8 = 1;
const STORE_SIZE_LIMIT: u64 = 128 * 1024;

#[derive(Default, Deserialize, Serialize)]
struct ToolMetricStore {
    version: u8,
    #[serde(default, deserialize_with = "deserialize_entries")]
    entries: Vec<ToolMetricEntry>,
}

pub async fn load() -> Result<Vec<ToolMetricEntry>, String> {
    let file = match tokio::fs::File::open(path()).await {
        Ok(file) => file,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(Vec::new()),
        Err(_) => return Err(unavailable()),
    };
    decode_bounded(file).await
}

pub async fn save(entries: &[ToolMetricEntry]) -> Result<(), String> {
    validate_entries(entries)?;
    let store = ToolMetricStore {
        version: STORE_VERSION,
        entries: entries.to_vec(),
    };
    let bytes = serde_json::to_vec(&store).map_err(|_| unavailable())?;
    if bytes.len() as u64 > STORE_SIZE_LIMIT {
        return Err(unavailable());
    }
    crate::services::private_store::atomic_write_async(path(), bytes).await
}

async fn decode_bounded(file: tokio::fs::File) -> Result<Vec<ToolMetricEntry>, String> {
    let mut bytes = Vec::new();
    file.take(STORE_SIZE_LIMIT.saturating_add(1))
        .read_to_end(&mut bytes)
        .await
        .map_err(|_| unavailable())?;
    if bytes.len() as u64 > STORE_SIZE_LIMIT {
        return Err(unavailable());
    }
    decode(&bytes)
}

fn decode(bytes: &[u8]) -> Result<Vec<ToolMetricEntry>, String> {
    let store: ToolMetricStore = serde_json::from_slice(bytes).map_err(|_| unavailable())?;
    if store.version != STORE_VERSION || validate_entries(&store.entries).is_err() {
        return Err(unavailable());
    }
    Ok(store.entries)
}

fn validate_entries(entries: &[ToolMetricEntry]) -> Result<(), String> {
    if entries.len() > MAX_TRACKED_TOOLS {
        return Err(unavailable());
    }
    let unique_names = entries
        .iter()
        .map(|entry| entry.name.as_str())
        .collect::<std::collections::BTreeSet<_>>();
    if unique_names.len() != entries.len() || entries.iter().any(invalid_entry) {
        return Err(unavailable());
    }
    Ok(())
}

fn invalid_entry(entry: &ToolMetricEntry) -> bool {
    let outcome_total = [
        entry.success,
        entry.running,
        entry.partial,
        entry.failed,
        entry.cancelled,
        entry.stopped,
    ]
    .into_iter()
    .fold(0_u64, u64::saturating_add);
    let error_total = [
        entry.errors.validation,
        entry.errors.permission,
        entry.errors.not_found,
        entry.errors.conflict,
        entry.errors.timeout,
        entry.errors.cancelled,
        entry.errors.unavailable,
        entry.errors.external,
        entry.errors.execution,
        entry.errors.internal,
    ]
    .into_iter()
    .fold(0_u64, u64::saturating_add);
    let permission_split = entry.user_denied.saturating_add(entry.policy_blocked);
    super::tool_metrics::validate_name(&entry.name).is_err()
        || entry.invocations == 0
        || entry.updated_at <= 0
        || outcome_total != entry.invocations
        || error_total > entry.invocations
        || permission_split > entry.errors.permission
        || permission_split > entry.failed
}

fn deserialize_entries<'de, D>(deserializer: D) -> Result<Vec<ToolMetricEntry>, D::Error>
where
    D: Deserializer<'de>,
{
    struct BoundedEntries;
    impl<'de> serde::de::Visitor<'de> for BoundedEntries {
        type Value = Vec<ToolMetricEntry>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            formatter.write_str("a bounded tool metric list")
        }

        fn visit_seq<A>(self, mut sequence: A) -> Result<Self::Value, A::Error>
        where
            A: serde::de::SeqAccess<'de>,
        {
            let mut entries = Vec::with_capacity(MAX_TRACKED_TOOLS);
            while let Some(entry) = sequence.next_element()? {
                if entries.len() >= MAX_TRACKED_TOOLS {
                    return Err(serde::de::Error::custom("tool metric limit exceeded"));
                }
                entries.push(entry);
            }
            Ok(entries)
        }
    }
    deserializer.deserialize_seq(BoundedEntries)
}

fn path() -> PathBuf {
    crate::services::paths::data_dir().join("tool-metrics.json")
}

fn unavailable() -> String {
    "Mesures d'outils indisponibles.".to_string()
}

#[cfg(test)]
#[path = "tool_metrics_store_tests.rs"]
mod tests;
