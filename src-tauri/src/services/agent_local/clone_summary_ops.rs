use std::collections::HashMap;
use std::sync::LazyLock;
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;

const MAX_ACTIVE_SUMMARIES: usize = 32;

static SUMMARY_OPS: LazyLock<Mutex<HashMap<String, CancellationToken>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

pub async fn register(operation_id: &str) -> Result<CancellationToken, String> {
    let mut map = SUMMARY_OPS.lock().await;
    if map.len() >= MAX_ACTIVE_SUMMARIES {
        return Err("Trop d'opérations en cours".into());
    }
    let token = CancellationToken::new();
    map.insert(operation_id.to_string(), token.clone());
    Ok(token)
}

pub async fn cancel(operation_id: &str) {
    let map = SUMMARY_OPS.lock().await;
    if let Some(token) = map.get(operation_id) {
        token.cancel();
    }
}

pub async fn finish(operation_id: &str) {
    SUMMARY_OPS.lock().await.remove(operation_id);
}
