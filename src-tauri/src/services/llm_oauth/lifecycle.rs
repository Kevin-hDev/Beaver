use std::sync::LazyLock;

use tokio::sync::{Mutex, MutexGuard};

use super::LlmOAuthProvider;

static LOCKS: LazyLock<[Mutex<()>; 2]> = LazyLock::new(|| [Mutex::new(()), Mutex::new(())]);

pub async fn lock(provider: LlmOAuthProvider) -> MutexGuard<'static, ()> {
    LOCKS[provider.index()].lock().await
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};

    #[tokio::test]
    async fn serializes_the_same_provider_lifecycle() {
        let first = lock(LlmOAuthProvider::Xai).await;
        let entered = std::sync::Arc::new(AtomicBool::new(false));
        let observed = entered.clone();
        let waiting = tokio::spawn(async move {
            let _second = lock(LlmOAuthProvider::Xai).await;
            observed.store(true, Ordering::SeqCst);
        });
        tokio::task::yield_now().await;
        assert!(!entered.load(Ordering::SeqCst));
        drop(first);
        waiting.await.unwrap();
        assert!(entered.load(Ordering::SeqCst));
    }
}
