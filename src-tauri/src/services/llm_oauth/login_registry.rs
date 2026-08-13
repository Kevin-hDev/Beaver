use std::sync::LazyLock;
use std::time::Duration;

use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;

use super::LlmOAuthProvider;
use crate::services::oauth_completion::{OAuthCompletion, OAuthCompletionOwner};

#[derive(Clone)]
struct ActiveLogin {
    cancel: CancellationToken,
    completion: OAuthCompletion<()>,
}

pub(super) struct RegisteredLogin {
    pub(super) cancel: CancellationToken,
    pub(super) completion: OAuthCompletionOwner<()>,
}

static ACTIVE: LazyLock<Mutex<[Option<ActiveLogin>; 2]>> =
    LazyLock::new(|| Mutex::new([None, None]));

pub async fn register(provider: LlmOAuthProvider) -> Result<RegisteredLogin, String> {
    let mut active = ACTIVE.lock().await;
    let slot = &mut active[provider.index()];
    if slot
        .as_ref()
        .is_some_and(|login| !login.completion.is_finished())
    {
        return Err("Connexion déjà en cours".to_string());
    }
    let token = CancellationToken::new();
    let (completion, observed) = OAuthCompletion::channel();
    *slot = Some(ActiveLogin {
        cancel: token.clone(),
        completion: observed,
    });
    Ok(RegisteredLogin {
        cancel: token,
        completion,
    })
}

pub async fn cancel(provider: LlmOAuthProvider) {
    let active = ACTIVE.lock().await[provider.index()].clone();
    if let Some(active) = active {
        active.cancel.cancel();
        let _ = tokio::time::timeout(Duration::from_secs(2), active.completion.wait()).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn limits_one_login_per_provider() {
        let first = register(LlmOAuthProvider::Xai).await.unwrap();
        assert!(register(LlmOAuthProvider::Xai).await.is_err());
        first.cancel.cancel();
        drop(first);
        assert!(register(LlmOAuthProvider::Xai).await.is_ok());
    }
}
