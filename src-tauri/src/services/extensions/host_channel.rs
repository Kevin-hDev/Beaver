use super::error_codes;
use super::types::MAX_MESSAGE_BYTES;
use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::process::ChildStdin;
use tokio::sync::{oneshot, Mutex};
use zeroize::Zeroizing;

pub type PendingSender = oneshot::Sender<Result<Value, String>>;
pub type PendingRequests = Arc<Mutex<HashMap<String, PendingSender>>>;
pub type SharedWriter = Arc<Mutex<ChildStdin>>;

pub async fn write(writer: &SharedWriter, message: &impl Serialize) -> Result<(), String> {
    let mut bytes = Zeroizing::new(
        serde_json::to_vec(message).map_err(|_| error_codes::REQUEST_INVALID.to_string())?,
    );
    if bytes.len() >= MAX_MESSAGE_BYTES {
        return Err(error_codes::REQUEST_TOO_LARGE.to_string());
    }
    bytes.push(b'\n');
    let mut stdin = writer.lock().await;
    stdin
        .write_all(&bytes)
        .await
        .map_err(|_| error_codes::HOST_UNAVAILABLE.to_string())?;
    stdin
        .flush()
        .await
        .map_err(|_| error_codes::HOST_UNAVAILABLE.to_string())
}

pub async fn fail_all(pending: &PendingRequests) {
    let requests = std::mem::take(&mut *pending.lock().await);
    for (_, sender) in requests {
        let _ = sender.send(Err(error_codes::HOST_UNAVAILABLE.to_string()));
    }
}
