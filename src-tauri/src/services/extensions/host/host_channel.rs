use super::error_codes;
use super::types::MAX_MESSAGE_BYTES;
use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex as StdMutex;
use tokio::io::AsyncWriteExt;
use tokio::process::ChildStdin;
use tokio::sync::{oneshot, Mutex};
use zeroize::Zeroizing;

pub type PendingSender = oneshot::Sender<Result<Value, String>>;
pub type PendingRequests = Arc<StdMutex<HashMap<String, PendingSender>>>;
pub type SharedWriter = Arc<Mutex<ChildStdin>>;
pub(super) const MAX_REQUEST_ID_CHARS: usize = 128;

pub(super) fn valid_request_id(value: &str) -> bool {
    !value.is_empty()
        && value.chars().count() <= MAX_REQUEST_ID_CHARS
        && !value.chars().any(char::is_control)
}

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

pub(super) fn insert(
    pending: &PendingRequests,
    id: String,
    sender: PendingSender,
) -> Result<(), String> {
    let mut requests = pending.lock().unwrap_or_else(|error| error.into_inner());
    if requests.len() >= super::types::MAX_PENDING_REQUESTS {
        return Err(error_codes::HOST_BUSY.to_string());
    }
    requests.insert(id, sender);
    Ok(())
}

pub(super) fn remove(pending: &PendingRequests, id: &str) -> Option<PendingSender> {
    pending
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .remove(id)
}

pub fn fail_all(pending: &PendingRequests) {
    let requests = std::mem::take(&mut *pending.lock().unwrap_or_else(|error| error.into_inner()));
    for (_, sender) in requests {
        let _ = sender.send(Err(error_codes::HOST_UNAVAILABLE.to_string()));
    }
}

#[cfg(test)]
pub(super) fn len(pending: &PendingRequests) -> usize {
    pending
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .len()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::process::Stdio;
    use tokio::io::{AsyncBufReadExt, BufReader};
    use tokio::process::Command;

    #[tokio::test]
    async fn oversized_response_is_rejected_without_corrupting_the_owned_channel() {
        let mut command = Command::new(which::which("node").unwrap());
        command
            .args(["-e", "process.stdin.pipe(process.stdout)"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .kill_on_drop(true);
        let (mut child, scope) = crate::services::owned_process::OwnedProcess::spawn_tokio_scoped(
            &mut command,
            crate::services::process_tree::ProcessKind::ExtensionHost,
        )
        .await
        .unwrap();
        let root_pid = child.id().unwrap();
        let writer = Arc::new(Mutex::new(child.stdin.take().unwrap()));
        let mut reader = BufReader::new(child.stdout.take().unwrap());

        write(&writer, &json!({"result": "before"})).await.unwrap();
        let mut line = String::new();
        reader.read_line(&mut line).await.unwrap();
        assert_eq!(
            serde_json::from_str::<Value>(line.trim()).unwrap(),
            json!({"result": "before"})
        );

        assert_eq!(
            write(&writer, &json!({"result": "x".repeat(MAX_MESSAGE_BYTES)})).await,
            Err(error_codes::REQUEST_TOO_LARGE.to_string())
        );

        write(&writer, &json!({"result": "after"})).await.unwrap();
        line.clear();
        reader.read_line(&mut line).await.unwrap();
        assert_eq!(
            serde_json::from_str::<Value>(line.trim()).unwrap(),
            json!({"result": "after"})
        );

        drop(writer);
        assert!(
            crate::services::process_tree::terminate_tokio_scoped(
                &mut child,
                crate::services::process_tree::ProcessKind::ExtensionHost,
                &scope,
                root_pid,
                std::time::Instant::now() + std::time::Duration::from_secs(5),
            )
            .await
        );
        assert!(child.wait().await.is_ok());
    }
}
