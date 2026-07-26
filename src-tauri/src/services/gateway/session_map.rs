use std::sync::LazyLock;

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use crate::services::paths::data_dir;

static MAP: LazyLock<Mutex<IndexMap<String, String>>> =
    LazyLock::new(|| Mutex::new(load_from_disk().unwrap_or_default()));

const MAP_VERSION: u8 = 2;

#[derive(Deserialize, Serialize)]
struct SessionMapFile {
    version: u8,
    entries: IndexMap<String, String>,
}

fn map_path() -> std::path::PathBuf {
    data_dir()
        .join("agent-sessions")
        .join("gateway-session-map.json")
}

fn load_from_disk() -> Option<IndexMap<String, String>> {
    let data = std::fs::read_to_string(map_path()).ok()?;
    parse_file(&data).ok()
}

fn parse_file(data: &str) -> Result<IndexMap<String, String>, serde_json::Error> {
    let parsed = serde_json::from_str::<SessionMapFile>(data);
    match parsed {
        Ok(file) if file.version == MAP_VERSION => Ok(file.entries),
        Ok(_) => Ok(IndexMap::new()),
        Err(error) => {
            if serde_json::from_str::<IndexMap<String, String>>(data).is_ok() {
                Ok(IndexMap::new())
            } else {
                Err(error)
            }
        }
    }
}

fn encode_file(map: &IndexMap<String, String>) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(&SessionMapFile {
        version: MAP_VERSION,
        entries: map.clone(),
    })
}

fn flush(map: &IndexMap<String, String>) -> Result<(), String> {
    let path = map_path();
    let json = encode_file(map).map_err(|_| "stockage des sessions impossible".to_string())?;
    crate::services::private_store::atomic_write(&path, json.as_bytes())
}

pub async fn find(channel_key: &str) -> Option<String> {
    let map = MAP.lock().await;
    map.get(channel_key).cloned()
}

pub async fn insert_bounded(
    channel_key: &str,
    session_id: &str,
    max_mappings: usize,
) -> Result<(), String> {
    let mut map = MAP.lock().await;
    let limit = max_mappings.max(1);
    if !map.contains_key(channel_key) {
        while map.len() >= limit {
            if map.shift_remove_index(0).is_none() {
                break;
            }
        }
    }
    map.insert(channel_key.to_string(), session_id.to_string());
    flush(&map)
}

#[cfg(test)]
mod tests {
    use super::*;

    static TEST_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

    #[tokio::test]
    async fn insert_and_find() {
        let _guard = TEST_LOCK.lock().await;
        let mut map = MAP.lock().await;
        map.insert("test/key".to_string(), "uuid-123".to_string());
        assert_eq!(map.get("test/key"), Some(&"uuid-123".to_string()));
        map.shift_remove("test/key");
    }

    #[tokio::test]
    async fn find_missing_returns_none() {
        let _guard = TEST_LOCK.lock().await;
        let map = MAP.lock().await;
        assert!(map.get("nonexistent/key").is_none());
    }

    #[tokio::test]
    async fn bounded_insert_evicts() {
        let _guard = TEST_LOCK.lock().await;
        let mut map = MAP.lock().await;
        map.clear();
        let _ = flush(&map);
        drop(map);

        insert_bounded("a", "1", 2).await.unwrap();
        insert_bounded("b", "2", 2).await.unwrap();
        insert_bounded("c", "3", 2).await.unwrap();

        let map = MAP.lock().await;
        assert_eq!(map.len(), 2);
        assert!(!map.contains_key("a"));
        assert_eq!(map.get("b"), Some(&"2".to_string()));
        assert_eq!(map.get("c"), Some(&"3".to_string()));
        let mut map = map;
        map.clear();
        let _ = flush(&map);
    }

    #[tokio::test]
    async fn load_from_disk_reads_flushed_map() {
        let _guard = TEST_LOCK.lock().await;
        let mut map = MAP.lock().await;
        map.clear();
        map.insert("reload/key".to_string(), "session-1".to_string());
        flush(&map).unwrap();
        drop(map);

        let loaded = load_from_disk().unwrap();
        assert_eq!(loaded.get("reload/key"), Some(&"session-1".to_string()));

        let mut map = MAP.lock().await;
        map.clear();
        let _ = flush(&map);
    }

    #[test]
    fn legacy_unversioned_map_is_not_reused() {
        let legacy = r#"{"slack/work/U123":"session-1"}"#;
        assert!(parse_file(legacy).unwrap().is_empty());
    }

    #[test]
    fn version_two_map_round_trips() {
        let mut entries = IndexMap::new();
        entries.insert("gateway:v2:abc".to_string(), "session-2".to_string());
        let encoded = encode_file(&entries).unwrap();

        assert_eq!(parse_file(&encoded).unwrap(), entries);
        assert!(encoded.contains("\"version\": 2"));
    }
}
