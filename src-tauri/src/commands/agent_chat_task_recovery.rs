use crate::services::agent_local::agent_loop_finish::CompletedStreamTurn;
use crate::services::agent_local::stream_recovery_apply::{
    recover_session, OwnerTerminal, StreamRecoveryMode,
};
use futures_util::FutureExt;
use tokio_util::sync::CancellationToken;

pub(crate) type SpawnedStreamTask = std::pin::Pin<
    Box<dyn std::future::Future<Output = Result<CompletedStreamTurn, String>> + Send + 'static>,
>;

pub(super) fn mascot_outcome(
    result: &Result<CompletedStreamTurn, String>,
) -> crate::services::mascot::MascotSessionOutcome {
    match result {
        Ok(_) => crate::services::mascot::MascotSessionOutcome::Success,
        Err(message) if message == "Annulé" => {
            crate::services::mascot::MascotSessionOutcome::Cancelled
        }
        Err(_) => crate::services::mascot::MascotSessionOutcome::Failed,
    }
}

pub(super) async fn guard<F>(
    future: F,
) -> Result<Result<CompletedStreamTurn, String>, Box<dyn std::any::Any + Send + 'static>>
where
    F: std::future::Future<Output = Result<CompletedStreamTurn, String>>,
{
    std::panic::AssertUnwindSafe(future).catch_unwind().await
}

pub(super) async fn finish(
    guarded: Result<Result<CompletedStreamTurn, String>, Box<dyn std::any::Any + Send + 'static>>,
    session_id: &str,
    request_id: &str,
    cancel: &CancellationToken,
    extension_events_admitted: bool,
) -> Result<CompletedStreamTurn, String> {
    let result = match guarded {
        Ok(Ok(completed)) => Ok(completed),
        Ok(Err(error)) => {
            let terminal = if error == "Annulé" {
                OwnerTerminal::Cancelled
            } else {
                OwnerTerminal::Failed { code: &error }
            };
            if recover_session(
                session_id,
                StreamRecoveryMode::Owner {
                    request_id,
                    terminal,
                },
            )
            .await
            .is_err()
            {
                log::warn!("stream_recovery_owner_unavailable");
            }
            Err(error)
        }
        Err(_) => {
            cancel.cancel();
            if recover_session(
                session_id,
                StreamRecoveryMode::Owner {
                    request_id,
                    terminal: OwnerTerminal::Failed {
                        code: "stream_error",
                    },
                },
            )
            .await
            .is_err()
            {
                log::warn!("stream_recovery_owner_unavailable");
            }
            Err("stream_error".to_string())
        }
    };
    if extension_events_admitted {
        super::session_events::emit_terminal(session_id, request_id, &result);
    }
    result
}

#[cfg(test)]
#[path = "agent_chat_task_recovery_tests.rs"]
mod tests;
