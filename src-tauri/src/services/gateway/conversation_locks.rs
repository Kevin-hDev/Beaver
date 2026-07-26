use std::collections::HashMap;
use std::sync::{Arc, Weak};
use std::time::Duration;

use tokio::sync::{Mutex, OwnedMutexGuard};

const CONVERSATION_WAIT_TIMEOUT: Duration = Duration::from_secs(300);

pub struct ConversationLocks {
    max_active: usize,
    wait_timeout: Duration,
    entries: Mutex<HashMap<String, Weak<Mutex<()>>>>,
}

impl ConversationLocks {
    pub fn new(max_active: usize) -> Self {
        Self::with_wait_timeout(max_active, CONVERSATION_WAIT_TIMEOUT)
    }

    fn with_wait_timeout(max_active: usize, wait_timeout: Duration) -> Self {
        Self {
            max_active: max_active.max(1),
            wait_timeout,
            entries: Mutex::new(HashMap::new()),
        }
    }

    pub async fn acquire(&self, key: &str) -> Result<OwnedMutexGuard<()>, String> {
        let lock = {
            let mut entries = self.entries.lock().await;
            entries.retain(|_, lock| lock.strong_count() > 0);
            if let Some(lock) = entries.get(key).and_then(Weak::upgrade) {
                lock
            } else {
                if entries.len() >= self.max_active {
                    return Err("trop de conversations actives".to_string());
                }
                let lock = Arc::new(Mutex::new(()));
                entries.insert(key.to_string(), Arc::downgrade(&lock));
                lock
            }
        };
        tokio::time::timeout(self.wait_timeout, lock.lock_owned())
            .await
            .map_err(|_| "conversation busy".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::ConversationLocks;
    use std::sync::Arc;
    use std::time::Duration;

    #[tokio::test]
    async fn same_conversation_is_serialized() {
        let locks = Arc::new(ConversationLocks::new(2));
        let first = locks.acquire("same").await.unwrap();
        let pending = {
            let locks = Arc::clone(&locks);
            tokio::spawn(async move { locks.acquire("same").await.unwrap() })
        };

        assert!(tokio::time::timeout(Duration::from_millis(20), pending)
            .await
            .is_err());
        drop(first);
        assert!(locks.acquire("same").await.is_ok());
    }

    #[tokio::test]
    async fn different_conversations_do_not_block_each_other() {
        let locks = ConversationLocks::new(2);
        let _first = locks.acquire("one").await.unwrap();

        assert!(
            tokio::time::timeout(Duration::from_millis(20), locks.acquire("two"))
                .await
                .unwrap()
                .is_ok()
        );
    }

    #[tokio::test]
    async fn active_lock_collection_is_bounded() {
        let locks = ConversationLocks::new(1);
        let _first = locks.acquire("one").await.unwrap();

        assert!(locks.acquire("two").await.is_err());
    }

    #[tokio::test]
    async fn waiting_for_a_busy_conversation_has_a_deadline() {
        let locks = ConversationLocks::with_wait_timeout(1, Duration::from_millis(20));
        let _first = locks.acquire("same").await.unwrap();

        let result = locks.acquire("same").await;

        assert!(result.is_err());
    }
}
