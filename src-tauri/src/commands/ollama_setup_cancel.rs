use std::sync::LazyLock;

use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;

static ACTIVE_SETUP_CANCEL: LazyLock<Mutex<Option<CancellationToken>>> =
    LazyLock::new(|| Mutex::new(None));

pub(crate) async fn register(token: CancellationToken) {
    *ACTIVE_SETUP_CANCEL.lock().await = Some(token);
}

pub(crate) async fn clear() {
    *ACTIVE_SETUP_CANCEL.lock().await = None;
}

pub(crate) async fn cancel_active() {
    if let Some(token) = ACTIVE_SETUP_CANCEL.lock().await.take() {
        token.cancel();
    }
}

pub(crate) fn cancelled_error() -> String {
    "ollama-setup-cancelled".to_string()
}

pub(crate) fn is_cancelled_error(err: &str) -> bool {
    err == "ollama-setup-cancelled"
}
