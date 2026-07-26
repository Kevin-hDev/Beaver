use std::sync::LazyLock;
use std::time::Duration;

use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;

use super::LlmOAuthProvider;

static ACTIVE: LazyLock<Mutex<[Option<CancellationToken>; 2]>> =
    LazyLock::new(|| Mutex::new([None, None]));

pub async fn register(provider: LlmOAuthProvider) -> Result<CancellationToken, String> {
    let mut active = ACTIVE.lock().await;
    let slot = &mut active[provider.index()];
    if slot.is_some() {
        return Err("Connexion déjà en cours".to_string());
    }
    let token = CancellationToken::new();
    *slot = Some(token.clone());
    Ok(token)
}

pub async fn release(provider: LlmOAuthProvider) {
    ACTIVE.lock().await[provider.index()] = None;
}

pub async fn cancel(provider: LlmOAuthProvider) {
    let token = ACTIVE.lock().await[provider.index()].clone();
    if let Some(token) = token {
        token.cancel();
        let _ = tokio::time::timeout(Duration::from_secs(2), async {
            loop {
                if ACTIVE.lock().await[provider.index()].is_none() {
                    break;
                }
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        })
        .await;
    }
}

pub async fn cancel_all() {
    let tokens = ACTIVE.lock().await.clone();
    for token in tokens.into_iter().flatten() {
        token.cancel();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn limits_one_login_per_provider() {
        let first = register(LlmOAuthProvider::Xai).await.unwrap();
        assert!(register(LlmOAuthProvider::Xai).await.is_err());
        first.cancel();
        release(LlmOAuthProvider::Xai).await;
    }
}
