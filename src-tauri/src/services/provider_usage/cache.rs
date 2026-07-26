use super::types::{RemoteData, CONNECTION_LIMIT};
use std::sync::LazyLock;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

const TTL: Duration = Duration::from_secs(60);

struct Entry {
    connection_id: String,
    generation: u64,
    value: RemoteData,
    inserted_at: Instant,
}

static CACHE: LazyLock<Mutex<Vec<Entry>>> = LazyLock::new(|| Mutex::new(Vec::new()));

pub async fn get(connection_id: &str) -> Option<RemoteData> {
    let mut cache = CACHE.lock().await;
    cache.retain(|entry| entry.inserted_at.elapsed() <= TTL);
    cache
        .iter()
        .find(|entry| {
            entry.connection_id == connection_id
                && super::credential_epoch::is_current(connection_id, entry.generation)
        })
        .map(|entry| entry.value.clone())
}

pub async fn put(connection_id: &str, generation: u64, value: RemoteData) {
    let mut cache = CACHE.lock().await;
    if let Some(entry) = cache
        .iter_mut()
        .find(|entry| entry.connection_id == connection_id)
    {
        entry.value = value;
        entry.generation = generation;
        entry.inserted_at = Instant::now();
        return;
    }
    if cache.len() >= CONNECTION_LIMIT {
        cache.remove(0);
    }
    cache.push(Entry {
        connection_id: connection_id.to_string(),
        generation,
        value,
        inserted_at: Instant::now(),
    });
}

pub async fn remove(connection_id: &str) {
    CACHE
        .lock()
        .await
        .retain(|entry| entry.connection_id != connection_id);
}

#[cfg(test)]
async fn clear() {
    CACHE.lock().await.clear();
}

#[cfg(test)]
async fn len() -> usize {
    CACHE.lock().await.len()
}

#[cfg(test)]
#[path = "cache_tests.rs"]
mod tests;
