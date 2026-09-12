use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{LazyLock, Mutex};

pub(crate) const MAX_STREAM_RECOVERY_LOGS: usize = 256;
type Key = (String, String);

static OWNERS: LazyLock<Mutex<HashMap<Key, u64>>> =
    LazyLock::new(|| Mutex::new(HashMap::with_capacity(MAX_STREAM_RECOVERY_LOGS)));
static NEXT_OWNER: AtomicU64 = AtomicU64::new(1);

pub(crate) struct OwnerLease {
    key: Key,
    owner: u64,
}

pub(crate) fn claim(session_id: &str, request_id: &str) -> Result<OwnerLease, String> {
    let key = (session_id.to_string(), request_id.to_string());
    let mut owners = OWNERS.lock().unwrap_or_else(|failure| failure.into_inner());
    if owners.contains_key(&key) || owners.len() >= MAX_STREAM_RECOVERY_LOGS {
        return Err(error());
    }
    let owner = NEXT_OWNER.fetch_add(1, Ordering::Relaxed);
    owners.insert(key.clone(), owner);
    Ok(OwnerLease { key, owner })
}

pub(crate) fn is_live(session_id: &str, request_id: &str) -> bool {
    OWNERS
        .lock()
        .unwrap_or_else(|failure| failure.into_inner())
        .contains_key(&(session_id.to_string(), request_id.to_string()))
}

impl Drop for OwnerLease {
    fn drop(&mut self) {
        let mut owners = OWNERS.lock().unwrap_or_else(|failure| failure.into_inner());
        if owners.get(&self.key) == Some(&self.owner) {
            owners.remove(&self.key);
        }
    }
}

fn error() -> String {
    "stream_recovery_unavailable".to_string()
}
