use super::core_bridge::CoreResponse;
use super::host_channel::{self, SharedWriter};
use super::host_reader::HostAuthority;
use super::protocol::{RpcError, RpcErrorBody, RpcResult};
use serde_json::Value;

pub(super) async fn spawn(
    id: String,
    method: String,
    params: Option<Value>,
    writer: &SharedWriter,
    work: &super::work_supervision::ExtensionWorkServices,
    authority: HostAuthority,
    channel_cancel: &tokio_util::sync::CancellationToken,
) -> Result<(), String> {
    let output = writer.clone();
    let task_cancel = channel_cancel.clone();
    let task_id = id.clone();
    let spawn = work.spawn_core_call(move |cancel| async move {
        let response = tokio::select! {
            biased;
            _ = task_cancel.cancelled() => return,
            _ = cancel.cancelled() => return,
            response = super::core_bridge::call(
                &authority.identity,
                &authority.api_level,
                &method,
                params.as_ref(),
            ) => response,
        };
        if task_cancel.is_cancelled() {
            return;
        }
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
        if channel_cancel.is_cancelled() {
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
