use super::core_bridge::CoreResponse;
use super::host_channel::{self, SharedWriter};
use super::protocol::{RpcError, RpcErrorBody, RpcResult};
use serde_json::Value;

pub(super) async fn spawn(
    id: String,
    method: String,
    params: Option<Value>,
    writer: &SharedWriter,
    work: &super::work_supervision::ExtensionWorkServices,
    context: super::call_context::ExtensionCallContext,
    reader_cancel: &tokio_util::sync::CancellationToken,
) -> Result<(), String> {
    let output = writer.clone();
    let task_cancel = context.revoked().clone();
    let spawn_cancel = task_cancel.clone();
    let reader_cancel = reader_cancel.clone();
    let task_id = id.clone();
    let spawn = work.spawn_core_call(move |cancel| async move {
        let response = tokio::select! {
            biased;
            _ = spawn_cancel.cancelled() => return,
            _ = reader_cancel.cancelled() => return,
            _ = cancel.cancelled() => return,
            response = super::core_bridge::call(
                &context,
                &method,
                params.as_ref(),
            ) => response,
        };
        if spawn_cancel.is_cancelled() || reader_cancel.is_cancelled() {
            return;
        }
        match response {
            Ok(CoreResponse::Json(result)) => {
                let _ = write_unrevoked(
                    &output,
                    &RpcResult {
                        jsonrpc: "2.0",
                        id: &task_id,
                        result,
                    },
                    &spawn_cancel,
                    &reader_cancel,
                )
                .await;
            }
            Ok(CoreResponse::Secret(secret)) => {
                let _ = write_unrevoked(
                    &output,
                    &RpcResult {
                        jsonrpc: "2.0",
                        id: &task_id,
                        result: secret.as_str(),
                    },
                    &spawn_cancel,
                    &reader_cancel,
                )
                .await;
            }
            Err(_) => {
                let _ = write_unrevoked(
                    &output,
                    &RpcError {
                        jsonrpc: "2.0",
                        id: &task_id,
                        error: RpcErrorBody {
                            code: -32601,
                            message: "core_method_unavailable",
                        },
                    },
                    &spawn_cancel,
                    &reader_cancel,
                )
                .await;
            }
        }
    });
    if spawn.is_err() {
        if task_cancel.is_cancelled() {
            return Err(super::error_codes::HOST_UNAVAILABLE.to_string());
        }
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

async fn write_unrevoked(
    writer: &SharedWriter,
    message: &impl serde::Serialize,
    revoked: &tokio_util::sync::CancellationToken,
    reader_cancel: &tokio_util::sync::CancellationToken,
) -> Result<(), String> {
    tokio::select! {
        biased;
        _ = revoked.cancelled() => Ok(()),
        _ = reader_cancel.cancelled() => Ok(()),
        result = host_channel::write(writer, message) => result,
    }
}
