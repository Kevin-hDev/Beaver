use super::host_channel::{self, PendingRequests, SharedWriter};
use super::host_load_tracker::HostLoadTracker;
use crate::services::work_registry::ServiceWorkCancellation;
use serde_json::Value;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::io::BufReader;
use tokio::process::ChildStdout;

pub(super) struct HostReaderChannel {
    pub(super) writer: SharedWriter,
    pub(super) pending: PendingRequests,
    pub(super) alive: Arc<AtomicBool>,
    pub(super) revoked: tokio_util::sync::CancellationToken,
    pub(super) reader_cancel: tokio_util::sync::CancellationToken,
    pub(super) load_tracker: Arc<HostLoadTracker>,
}

#[derive(Clone)]
pub(super) struct HostAuthority {
    pub(super) identity: super::host_identity::HostIdentity,
    pub(super) generation: u64,
}

struct HostReaderContext<'a> {
    writer: &'a SharedWriter,
    pending: &'a PendingRequests,
    load_tracker: &'a HostLoadTracker,
    alive: &'a AtomicBool,
    revoked: &'a tokio_util::sync::CancellationToken,
    reader_cancel: &'a tokio_util::sync::CancellationToken,
    #[cfg(test)]
    call_context: Option<super::call_context::ExtensionCallContext>,
}

pub async fn run(
    stdout: ChildStdout,
    channel: HostReaderChannel,
    work: super::work_supervision::ExtensionWorkServices,
    cancellation: ServiceWorkCancellation,
    authority: HostAuthority,
) {
    let mut reader = BufReader::new(stdout);
    loop {
        let bytes = tokio::select! {
            biased;
            _ = channel.revoked.cancelled() => break,
            _ = channel.reader_cancel.cancelled() => break,
            _ = cancellation.cancelled() => break,
            line = super::host_reader_line::read_bounded_line(&mut reader) => match line {
                Ok(line) => line,
                Err(_) => break,
            },
        };
        let context = HostReaderContext {
            writer: &channel.writer,
            pending: &channel.pending,
            load_tracker: &channel.load_tracker,
            alive: &channel.alive,
            revoked: &channel.revoked,
            reader_cancel: &channel.reader_cancel,
            #[cfg(test)]
            call_context: None,
        };
        if receive_bound(&bytes, &context, &work, &authority)
            .await
            .is_err()
        {
            break;
        }
    }
    channel.alive.store(false, Ordering::Release);
    channel.reader_cancel.cancel();
    channel.load_tracker.clear().await;
    host_channel::fail_all(&channel.pending).await;
}

async fn receive_bound(
    bytes: &[u8],
    context: &HostReaderContext<'_>,
    work: &super::work_supervision::ExtensionWorkServices,
    authority: &HostAuthority,
) -> Result<(), String> {
    if !context.alive.load(Ordering::Acquire)
        || context.revoked.is_cancelled()
        || context.reader_cancel.is_cancelled()
    {
        return Err(super::error_codes::HOST_UNAVAILABLE.to_string());
    }
    let message: Value = serde_json::from_slice(bytes)
        .map_err(|_| "Réponse de l'hôte d'extensions invalide.".to_string())?;
    super::validation::message(&message)?;
    let object = super::protocol::envelope(&message)?;
    if let Some(method) = object.get("method").and_then(Value::as_str) {
        if object.get("id").is_none() {
            return receive_notification(method, object.get("params"), context.load_tracker).await;
        }
        let id = object
            .get("id")
            .and_then(Value::as_str)
            .ok_or_else(|| "Réponse de l'hôte d'extensions invalide.".to_string())?;
        let params = object.get("params").cloned();
        let call_context = context_for_call(context, authority).await?;
        return super::host_core_call::spawn(
            id.to_string(),
            method.to_string(),
            params,
            context.writer,
            work,
            call_context,
            context.reader_cancel,
        )
        .await;
    }
    let id = object
        .get("id")
        .and_then(Value::as_str)
        .ok_or_else(|| "Réponse de l'hôte d'extensions invalide.".to_string())?;
    let Some(sender) = context.pending.lock().await.remove(id) else {
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

#[cfg(test)]
async fn receive(
    bytes: &[u8],
    writer: &SharedWriter,
    pending: &PendingRequests,
    load_tracker: &HostLoadTracker,
    work: &super::work_supervision::ExtensionWorkServices,
) -> Result<(), String> {
    let alive = AtomicBool::new(true);
    let revoked = tokio_util::sync::CancellationToken::new();
    let reader_cancel = tokio_util::sync::CancellationToken::new();
    let context = HostReaderContext {
        writer,
        pending,
        load_tracker,
        alive: &alive,
        revoked: &revoked,
        reader_cancel: &reader_cancel,
        call_context: Some(super::call_context::ExtensionCallContext::for_test(
            super::host_identity::HostIdentity::Official,
            super::types::ExtensionApiLevel::Stable,
        )),
    };
    let authority = HostAuthority {
        identity: super::host_identity::HostIdentity::Official,
        generation: 1,
    };
    receive_bound(bytes, &context, work, &authority).await
}

async fn context_for_call(
    _context: &HostReaderContext<'_>,
    authority: &HostAuthority,
) -> Result<super::call_context::ExtensionCallContext, String> {
    #[cfg(test)]
    if let Some(call_context) = &_context.call_context {
        return Ok(call_context.clone());
    }
    super::runtime::call_context(&authority.identity, authority.generation).await
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

#[cfg(test)]
#[path = "host_reader_tests.rs"]
mod tests;
