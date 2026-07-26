use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use tokio::sync::Mutex;

use super::limits::{MAX_ANALYSIS_INDEX_BYTES, MAX_BACKTEST_RESULTS, MAX_STORED_ANALYSES};
use super::storage_paths::{index_path, validate_analysis_id, validate_analysis_name};
use super::types::ForecastAnalysisMeta;

const INDEX_SCHEMA_VERSION: u32 = 1;
static INDEX_LOCK: Mutex<()> = Mutex::const_new(());

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ForecastIndex {
    schema_version: u32,
    entries: Vec<ForecastAnalysisMeta>,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum StoredIndex {
    Current(ForecastIndex),
    Legacy(Vec<ForecastAnalysisMeta>),
}

struct IndexState {
    entries: Vec<ForecastAnalysisMeta>,
    needs_hydration: bool,
}

pub(super) async fn list() -> Result<Vec<ForecastAnalysisMeta>, String> {
    let _guard = INDEX_LOCK.lock().await;
    let mut state = read().await?;
    if state.needs_hydration {
        hydrate_legacy(&mut state.entries).await;
        write(&state.entries).await?;
    }
    Ok(state.entries)
}

pub(super) async fn entries() -> Result<Vec<ForecastAnalysisMeta>, String> {
    let _guard = INDEX_LOCK.lock().await;
    Ok(read().await?.entries)
}

pub(super) async fn upsert(meta: ForecastAnalysisMeta) -> Result<Vec<String>, String> {
    validate_entry(&meta)?;
    let _guard = INDEX_LOCK.lock().await;
    let mut state = read().await?;
    if state.needs_hydration {
        hydrate_legacy(&mut state.entries).await;
    }
    let mut entries = state.entries;
    let removed_ids = upsert_entries(&mut entries, meta);
    write(&entries).await?;
    Ok(removed_ids)
}

pub(super) async fn remove(id: &str) -> Result<(), String> {
    let _guard = INDEX_LOCK.lock().await;
    let mut state = read().await?;
    if state.needs_hydration {
        hydrate_legacy(&mut state.entries).await;
    }
    let mut entries = state.entries;
    entries.retain(|entry| entry.id != id);
    write(&entries).await
}

async fn read() -> Result<IndexState, String> {
    let data = match super::storage_io::read_bounded(&index_path(), MAX_ANALYSIS_INDEX_BYTES).await
    {
        Ok(data) => data,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(IndexState {
                entries: Vec::new(),
                needs_hydration: false,
            });
        }
        Err(_) => return Err("Index forecast indisponible".into()),
    };
    parse(&data)
}

fn parse(data: &[u8]) -> Result<IndexState, String> {
    let stored: StoredIndex =
        serde_json::from_slice(data).map_err(|_| "Index forecast corrompu".to_string())?;
    let (entries, needs_hydration) = match stored {
        StoredIndex::Current(index) if index.schema_version == INDEX_SCHEMA_VERSION => {
            (index.entries, false)
        }
        StoredIndex::Legacy(entries) => (entries, true),
        StoredIndex::Current(_) => return Err("Index forecast incompatible".into()),
    };
    validate_entries(&entries)?;
    Ok(IndexState {
        entries,
        needs_hydration,
    })
}

async fn write(entries: &[ForecastAnalysisMeta]) -> Result<(), String> {
    validate_entries(entries)?;
    let json = serde_json::to_vec_pretty(&ForecastIndex {
        schema_version: INDEX_SCHEMA_VERSION,
        entries: entries.to_vec(),
    })
    .map_err(|_| "Index forecast invalide".to_string())?;
    if json.len() > MAX_ANALYSIS_INDEX_BYTES {
        return Err("Index forecast trop volumineux".into());
    }
    crate::services::private_store::atomic_write_async(index_path(), json)
        .await
        .map_err(|_| "Finalisation index forecast échouée".to_string())
}

async fn hydrate_legacy(entries: &mut [ForecastAnalysisMeta]) {
    for meta in entries {
        if let Ok(analysis) = super::storage::load(&meta.id).await {
            meta.scenarios_count = analysis.scenarios.len();
        }
    }
}

fn validate_entries(entries: &[ForecastAnalysisMeta]) -> Result<(), String> {
    if entries.len() > MAX_STORED_ANALYSES {
        return Err("Index forecast trop volumineux".into());
    }
    let mut ids = BTreeSet::new();
    for entry in entries {
        validate_entry(entry)?;
        if !ids.insert(entry.id.as_str()) {
            return Err("Index forecast corrompu".into());
        }
    }
    Ok(())
}

fn validate_entry(entry: &ForecastAnalysisMeta) -> Result<(), String> {
    validate_analysis_id(&entry.id).map_err(|_| "Index forecast corrompu".to_string())?;
    validate_analysis_name(&entry.name).map_err(|_| "Index forecast corrompu".to_string())?;
    if entry.session_id.as_deref().is_some_and(|id| {
        crate::services::agent_local::session_store::validate_session_id(id).is_err()
    }) || entry
        .backtest
        .as_ref()
        .is_some_and(|backtest| backtest.results.len() > MAX_BACKTEST_RESULTS)
    {
        return Err("Index forecast corrompu".into());
    }
    Ok(())
}

fn upsert_entries(
    entries: &mut Vec<ForecastAnalysisMeta>,
    meta: ForecastAnalysisMeta,
) -> Vec<String> {
    if let Some(position) = entries.iter().position(|entry| entry.id == meta.id) {
        entries[position] = meta;
        return Vec::new();
    }
    entries.push(meta);
    let overflow = entries.len().saturating_sub(MAX_STORED_ANALYSES);
    entries.drain(0..overflow).map(|entry| entry.id).collect()
}

#[cfg(test)]
#[path = "storage_index_tests.rs"]
mod tests;
