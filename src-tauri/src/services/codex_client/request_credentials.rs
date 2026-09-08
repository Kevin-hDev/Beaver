use crate::services::codex_oauth::store::CodexTokens;

pub(in crate::services::codex_client) struct RequestCredentials {
    pub(super) tokens: CodexTokens,
    #[cfg(test)]
    drop_observer: Option<std::sync::Arc<std::sync::atomic::AtomicBool>>,
}

impl RequestCredentials {
    pub(super) fn new(tokens: CodexTokens) -> Self {
        Self {
            tokens,
            #[cfg(test)]
            drop_observer: None,
        }
    }

    #[cfg(test)]
    pub(in crate::services::codex_client) fn observed(
        tokens: CodexTokens,
        observer: std::sync::Arc<std::sync::atomic::AtomicBool>,
    ) -> Self {
        Self {
            tokens,
            drop_observer: Some(observer),
        }
    }
}

impl Drop for RequestCredentials {
    fn drop(&mut self) {
        #[cfg(test)]
        if let Some(observer) = &self.drop_observer {
            observer.store(true, std::sync::atomic::Ordering::SeqCst);
        }
    }
}
