use serde::{Deserialize, Deserializer, Serialize};
use std::io::Read;
use std::path::PathBuf;

use super::request_journal::{ProviderRequestMetric, REQUEST_LIMIT};

const STORE_VERSION: u8 = 2;
const ZERO_BASED_STORE_VERSION: u8 = 1;
const STORE_SIZE_LIMIT: u64 = 8 * 1024 * 1024;

#[derive(Default, Serialize, Deserialize)]
pub(super) struct RequestStore {
    version: u8,
    #[serde(default, deserialize_with = "deserialize_entries")]
    pub entries: Vec<ProviderRequestMetric>,
}

pub(super) fn load() -> Result<RequestStore, String> {
    match std::fs::File::open(path()) {
        Ok(file) => decode_bounded(file),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(RequestStore::default()),
        Err(_) => Err(unavailable()),
    }
}

pub(super) fn save(store: &RequestStore) -> Result<(), String> {
    let mut normalized = RequestStore {
        version: STORE_VERSION,
        entries: store.entries.clone(),
    };
    super::request_journal::prune(&mut normalized.entries);
    let bytes = serde_json::to_vec(&normalized).map_err(|_| unavailable())?;
    if bytes.len() as u64 > STORE_SIZE_LIMIT {
        return Err(unavailable());
    }
    crate::services::private_store::atomic_write(&path(), &bytes)
}

fn path() -> PathBuf {
    crate::services::paths::data_dir().join("provider-request-metrics.json")
}

fn decode_bounded(file: std::fs::File) -> Result<RequestStore, String> {
    let mut bytes = Vec::new();
    file.take(STORE_SIZE_LIMIT.saturating_add(1))
        .read_to_end(&mut bytes)
        .map_err(|_| unavailable())?;
    if bytes.len() as u64 > STORE_SIZE_LIMIT {
        return Err(unavailable());
    }
    let mut store: RequestStore = serde_json::from_slice(&bytes).map_err(|_| unavailable())?;
    migrate_version(&mut store)?;
    Ok(store)
}

fn migrate_version(store: &mut RequestStore) -> Result<(), String> {
    match store.version {
        STORE_VERSION => Ok(()),
        ZERO_BASED_STORE_VERSION => {
            for metric in &mut store.entries {
                metric.turn = metric
                    .turn
                    .map(|turn| turn.checked_add(1).ok_or_else(unavailable))
                    .transpose()?;
            }
            store.version = STORE_VERSION;
            Ok(())
        }
        _ => Err(unavailable()),
    }
}

fn deserialize_entries<'de, D>(deserializer: D) -> Result<Vec<ProviderRequestMetric>, D::Error>
where
    D: Deserializer<'de>,
{
    struct BoundedEntries;
    impl<'de> serde::de::Visitor<'de> for BoundedEntries {
        type Value = Vec<ProviderRequestMetric>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            formatter.write_str("a bounded provider request list")
        }

        fn visit_seq<A>(self, mut sequence: A) -> Result<Self::Value, A::Error>
        where
            A: serde::de::SeqAccess<'de>,
        {
            let mut entries = Vec::with_capacity(REQUEST_LIMIT);
            while let Some(metric) = sequence.next_element()? {
                if entries.len() >= REQUEST_LIMIT {
                    return Err(serde::de::Error::custom("provider request limit exceeded"));
                }
                entries.push(metric);
            }
            Ok(entries)
        }
    }
    deserializer.deserialize_seq(BoundedEntries)
}

fn unavailable() -> String {
    "Mesures provider indisponibles".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_one_turns_are_migrated_without_touching_missing_values() {
        let mut store = RequestStore {
            version: ZERO_BASED_STORE_VERSION,
            entries: vec![
                ProviderRequestMetric {
                    turn: Some(0),
                    ..Default::default()
                },
                ProviderRequestMetric {
                    turn: Some(4),
                    ..Default::default()
                },
                ProviderRequestMetric::default(),
            ],
        };

        migrate_version(&mut store).unwrap();

        assert_eq!(store.version, STORE_VERSION);
        assert_eq!(store.entries[0].turn, Some(1));
        assert_eq!(store.entries[1].turn, Some(5));
        assert_eq!(store.entries[2].turn, None);
    }

    #[test]
    fn current_and_unknown_versions_are_not_silently_rewritten() {
        let mut current = RequestStore {
            version: STORE_VERSION,
            entries: Vec::new(),
        };
        assert!(migrate_version(&mut current).is_ok());

        current.version = 99;
        assert!(migrate_version(&mut current).is_err());
    }
}
