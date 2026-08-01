use serde::{Deserialize, Deserializer, Serialize};
use std::io::Read;
use std::path::PathBuf;

use super::request_journal::{ProviderRequestMetric, REQUEST_LIMIT};

const STORE_VERSION: u8 = 1;
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
    let store: RequestStore = serde_json::from_slice(&bytes).map_err(|_| unavailable())?;
    if store.version != STORE_VERSION {
        return Err(unavailable());
    }
    Ok(store)
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
