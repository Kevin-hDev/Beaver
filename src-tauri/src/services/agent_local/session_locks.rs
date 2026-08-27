use std::collections::HashMap;
use std::sync::{Arc, LazyLock};
use tokio::sync::Mutex;

const MAX_SESSION_LOCKS: usize = 512;

struct SessionLockEntry {
    lock: Arc<Mutex<()>>,
    last_used: u64,
}

#[derive(Default)]
struct SessionLockState {
    next_tick: u64,
    entries: HashMap<String, SessionLockEntry>,
}

static SESSION_LOCKS: LazyLock<Mutex<SessionLockState>> =
    LazyLock::new(|| Mutex::new(SessionLockState::default()));

pub(crate) struct AdmissionLease {
    session_id: String,
    _guard: tokio::sync::OwnedMutexGuard<()>,
}

impl AdmissionLease {
    pub(crate) fn session_id(&self) -> &str {
        &self.session_id
    }
}

pub(crate) async fn lock_session(id: &str) -> Arc<Mutex<()>> {
    let mut state = SESSION_LOCKS.lock().await;
    state.next_tick = state.next_tick.saturating_add(1);
    let tick = state.next_tick;
    let lock = state
        .entries
        .entry(id.to_string())
        .and_modify(|entry| entry.last_used = tick)
        .or_insert_with(|| SessionLockEntry {
            lock: Arc::new(Mutex::new(())),
            last_used: tick,
        })
        .lock
        .clone();
    evict_unused_locks(&mut state);
    lock
}

pub(crate) async fn acquire_admission_lease(id: &str) -> AdmissionLease {
    let guard = lock_session(id).await.lock_owned().await;
    AdmissionLease {
        session_id: id.to_string(),
        _guard: guard,
    }
}

pub async fn remove_session_lock(id: &str) {
    let mut state = SESSION_LOCKS.lock().await;
    let is_unused = state
        .entries
        .get(id)
        .is_some_and(|entry| Arc::strong_count(&entry.lock) == 1);
    if is_unused {
        state.entries.remove(id);
    }
}

pub async fn cancel_with_lock(id: &str, token: &tokio_util::sync::CancellationToken) {
    let lock = lock_session(id).await;
    let _guard = lock.lock().await;
    token.cancel();
}

fn evict_unused_locks(state: &mut SessionLockState) {
    while state.entries.len() > MAX_SESSION_LOCKS {
        let Some(oldest_unused) = state
            .entries
            .iter()
            .filter(|(_, entry)| Arc::strong_count(&entry.lock) == 1)
            .min_by_key(|(_, entry)| entry.last_used)
            .map(|(id, _)| id.clone())
        else {
            break;
        };
        state.entries.remove(&oldest_unused);
    }
}

#[cfg(test)]
pub(crate) async fn has_session_lock_for_test(id: &str) -> bool {
    SESSION_LOCKS.lock().await.entries.contains_key(id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[tokio::test]
    async fn remove_session_lock_drops_entry() {
        let id = Uuid::new_v4().to_string();

        let lock = lock_session(&id).await;
        assert!(has_session_lock_for_test(&id).await);

        drop(lock);
        remove_session_lock(&id).await;
        assert!(!has_session_lock_for_test(&id).await);
    }

    #[tokio::test]
    async fn admission_lease_owns_the_guard_and_exact_session_identity() {
        let id = Uuid::new_v4().to_string();

        let lease = acquire_admission_lease(&id).await;

        assert_eq!(lease.session_id(), id);
        assert!(has_session_lock_for_test(&id).await);
    }
}
