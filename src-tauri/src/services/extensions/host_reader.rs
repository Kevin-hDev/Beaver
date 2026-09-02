use super::core_bridge::CoreResponse;
use super::host_channel::{self, PendingRequests, SharedWriter};
use super::host_load_tracker::HostLoadTracker;
use super::protocol::{RpcError, RpcErrorBody, RpcResult};
use super::types::MAX_MESSAGE_BYTES;
use crate::services::work_registry::ServiceWorkCancellation;
use serde_json::Value;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::ChildStdout;

pub async fn run(
    stdout: ChildStdout,
    writer: SharedWriter,
    pending: PendingRequests,
    alive: Arc<AtomicBool>,
    load_tracker: Arc<HostLoadTracker>,
    work: super::work_supervision::ExtensionWorkServices,
    cancellation: ServiceWorkCancellation,
) {
    let mut reader = BufReader::new(stdout);
    loop {
        let bytes = tokio::select! {
            _ = cancellation.cancelled() => break,
            line = read_bounded_line(&mut reader) => match line {
                Ok(line) => line,
                Err(_) => break,
            },
        };
        if receive(&bytes, &writer, &pending, &load_tracker, &work)
            .await
            .is_err()
        {
            break;
        }
    }
    alive.store(false, Ordering::Release);
    load_tracker.clear().await;
    host_channel::fail_all(&pending).await;
}

async fn receive(
    bytes: &[u8],
    writer: &SharedWriter,
    pending: &PendingRequests,
    load_tracker: &HostLoadTracker,
    work: &super::work_supervision::ExtensionWorkServices,
) -> Result<(), String> {
    let message: Value = serde_json::from_slice(bytes)
        .map_err(|_| "Réponse de l'hôte d'extensions invalide.".to_string())?;
    super::validation::message(&message)?;
    let object = super::protocol::envelope(&message)?;
    if let Some(method) = object.get("method").and_then(Value::as_str) {
        if object.get("id").is_none() {
            return receive_notification(method, object.get("params"), load_tracker).await;
        }
        let id = object
            .get("id")
            .and_then(Value::as_str)
            .ok_or_else(|| "Réponse de l'hôte d'extensions invalide.".to_string())?;
        let params = object.get("params").cloned();
        return spawn_core_call(id.to_string(), method.to_string(), params, writer, work).await;
    }
    let id = object
        .get("id")
        .and_then(Value::as_str)
        .ok_or_else(|| "Réponse de l'hôte d'extensions invalide.".to_string())?;
    let Some(sender) = pending.lock().await.remove(id) else {
        return Ok(());
    };
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

async fn receive_notification(
    method: &str,
    params: Option<&Value>,
    load_tracker: &HostLoadTracker,
) -> Result<(), String> {
    if method != "host.load.stage" {
        return Err("Réponse de l'hôte d'extensions invalide.".to_string());
    }
    let params = params
        .and_then(Value::as_object)
        .filter(|params| params.len() == 1)
        .ok_or_else(|| "Réponse de l'hôte d'extensions invalide.".to_string())?;
    let stage = params
        .get("stage")
        .and_then(Value::as_str)
        .ok_or_else(|| "Réponse de l'hôte d'extensions invalide.".to_string())?;
    load_tracker.advance(stage).await.map(|_| ())
}

async fn spawn_core_call(
    id: String,
    method: String,
    params: Option<Value>,
    writer: &SharedWriter,
    work: &super::work_supervision::ExtensionWorkServices,
) -> Result<(), String> {
    let output = writer.clone();
    let task_id = id.clone();
    let spawn = work.spawn_core_call(move |cancel| async move {
        let response = tokio::select! {
            _ = cancel.cancelled() => return,
            response = super::core_bridge::call(&method, params.as_ref()) => response,
        };
        match response {
            Ok(CoreResponse::Json(result)) => {
                let _ = host_channel::write(
                    &output,
                    &RpcResult {
                        jsonrpc: "2.0",
                        id: &task_id,
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
                        id: &task_id,
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
                        id: &task_id,
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
    if spawn.is_err() {
        return host_channel::write(
            writer,
            &RpcError {
                jsonrpc: "2.0",
                id: &id,
                error: RpcErrorBody {
                    code: -32000,
                    message: "core_busy",
                },
            },
        )
        .await;
    }
    Ok(())
}

#[cfg(test)]
#[path = "host_reader_tests.rs"]
mod tests;

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
