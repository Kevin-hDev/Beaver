use super::core_bridge::CoreResponse;
use super::host_channel::{self, PendingRequests, SharedWriter};
use super::protocol::{RpcError, RpcErrorBody, RpcResult};
use super::types::MAX_MESSAGE_BYTES;
use serde_json::Value;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::ChildStdout;
use tokio::sync::Semaphore;

const MAX_CORE_REQUESTS: usize = 64;

pub async fn run(
    stdout: ChildStdout,
    writer: SharedWriter,
    pending: PendingRequests,
    alive: Arc<AtomicBool>,
) {
    let mut reader = BufReader::new(stdout);
    let core_limit = Arc::new(Semaphore::new(MAX_CORE_REQUESTS));
    while let Ok(bytes) = read_bounded_line(&mut reader).await {
        if receive(&bytes, &writer, &pending, &core_limit)
            .await
            .is_err()
        {
            break;
        }
    }
    alive.store(false, Ordering::Release);
    host_channel::fail_all(&pending).await;
}

async fn receive(
    bytes: &[u8],
    writer: &SharedWriter,
    pending: &PendingRequests,
    core_limit: &Arc<Semaphore>,
) -> Result<(), String> {
    let message: Value = serde_json::from_slice(bytes)
        .map_err(|_| "Réponse de l'hôte d'extensions invalide.".to_string())?;
    super::validation::message(&message)?;
    let object = super::protocol::envelope(&message)?;
    let id = object
        .get("id")
        .and_then(Value::as_str)
        .ok_or_else(|| "Réponse de l'hôte d'extensions invalide.".to_string())?;
    if let Some(method) = object.get("method").and_then(Value::as_str) {
        let params = object.get("params").cloned();
        return spawn_core_call(
            id.to_string(),
            method.to_string(),
            params,
            writer,
            core_limit,
        )
        .await;
    }
    let sender = pending
        .lock()
        .await
        .remove(id)
        .ok_or_else(|| "Réponse de l'hôte d'extensions invalide.".to_string())?;
    let result = if object.contains_key("error") {
        Err("L'hôte d'extensions a refusé la requête.".to_string())
    } else {
        object
            .get("result")
            .cloned()
            .ok_or_else(|| "Réponse de l'hôte d'extensions invalide.".to_string())
    };
    let _ = sender.send(result);
    Ok(())
}

async fn spawn_core_call(
    id: String,
    method: String,
    params: Option<Value>,
    writer: &SharedWriter,
    core_limit: &Arc<Semaphore>,
) -> Result<(), String> {
    let permit = core_limit
        .clone()
        .try_acquire_owned()
        .map_err(|_| "Trop de requêtes d'extension.".to_string())?;
    let output = writer.clone();
    tokio::spawn(async move {
        let _permit = permit;
        match super::core_bridge::call(&method, params.as_ref()).await {
            Ok(CoreResponse::Json(result)) => {
                let _ = host_channel::write(
                    &output,
                    &RpcResult {
                        jsonrpc: "2.0",
                        id: &id,
                        result,
                    },
                )
                .await;
            }
            Ok(CoreResponse::Secret(secret)) => {
                let _ = host_channel::write(
                    &output,
                    &RpcResult {
                        jsonrpc: "2.0",
                        id: &id,
                        result: secret.as_str(),
                    },
                )
                .await;
            }
            Err(()) => {
                let _ = host_channel::write(
                    &output,
                    &RpcError {
                        jsonrpc: "2.0",
                        id: &id,
                        error: RpcErrorBody {
                            code: -32601,
                            message: "core_method_unavailable",
                        },
                    },
                )
                .await;
            }
        }
    });
    Ok(())
}

async fn read_bounded_line(reader: &mut BufReader<ChildStdout>) -> Result<Vec<u8>, String> {
    let mut line = Vec::new();
    loop {
        let available = reader
            .fill_buf()
            .await
            .map_err(|_| "Hôte d'extensions indisponible.".to_string())?;
        if available.is_empty() {
            return Err("L'hôte d'extensions s'est arrêté.".to_string());
        }
        let take = available
            .iter()
            .position(|byte| *byte == b'\n')
            .map_or(available.len(), |index| index + 1);
        if line.len().saturating_add(take) > MAX_MESSAGE_BYTES {
            return Err("Réponse de l'hôte d'extensions trop volumineuse.".to_string());
        }
        line.extend_from_slice(&available[..take]);
        reader.consume(take);
        if line.last() == Some(&b'\n') {
            line.pop();
            return Ok(line);
        }
    }
}
