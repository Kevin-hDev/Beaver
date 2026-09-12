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
) -> Result<CompletedStreamTurn, String> {
    match guarded {
        Ok(Ok(completed)) => return Ok(completed),
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
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn stream_recovery_process_converts_a_worker_panic_and_restores_its_token() {
        use crate::services::agent_local::stream_recovery_log::StreamRecoveryLog;
        use crate::services::agent_local::stream_recovery_record::*;
        let mut session = crate::services::agent_local::session_store::create_full(
            "Panic recovery",
            "model",
            "openai",
            false,
            None,
        )
        .await
        .unwrap();
        let request_id = uuid::Uuid::new_v4().to_string();
        let header = StreamRecoveryHeader {
            version: STREAM_RECOVERY_VERSION,
            process_instance_id: process_instance_id().into(),
            session_id: session.id.clone(),
            request_id: request_id.clone(),
            turn_id: uuid::Uuid::new_v4().to_string(),
            user_message_id: uuid::Uuid::new_v4().to_string(),
            assistant_message_id: uuid::Uuid::new_v4().to_string(),
            subagent_owner: None,
            created_at: chrono::Utc::now(),
        };
        session.messages.push(user_message(&header));
        crate::services::agent_local::session_store::save(&session)
            .await
            .unwrap();
        let cancel = CancellationToken::new();
        let (log, lease) = StreamRecoveryLog::create(header, cancel.clone())
            .await
            .unwrap();
        log.record_event(RecoverableStreamEvent::Token {
            content: "visible before panic".into(),
            phase: None,
        })
        .unwrap();
        drop(log);
        drop(lease);
        let guarded = guard(async {
            panic!("private panic payload");
            #[allow(unreachable_code)]
            Ok(CompletedStreamTurn::compression(Vec::new()))
        })
        .await;

        let error = match finish(guarded, &session.id, &request_id, &cancel).await {
            Err(error) => error,
            Ok(_) => panic!("panic must become an error"),
        };
        assert_eq!(error, "stream_error");
        assert!(cancel.is_cancelled());
        assert!(
            crate::services::agent_local::session_store::get(&session.id)
                .await
                .unwrap()
                .messages
                .iter()
                .any(|message| message.content == "visible before panic")
        );
        crate::services::agent_local::session_store::delete_one(&session.id)
            .await
            .unwrap();
    }

    fn user_message(
        header: &crate::services::agent_local::stream_recovery_record::StreamRecoveryHeader,
    ) -> crate::services::agent_local::types_message::AgentMessage {
        crate::services::agent_local::types_message::AgentMessage {
            id: header.user_message_id.clone(),
            turn_id: header.turn_id.clone(),
            role: "user".into(),
            content: "continue".into(),
            message_kind: None,
            thinking: None,
            tool_calls: None,
            tool_name: None,
            tool_call_id: None,
            continuation: None,
            replay_source: None,
            tool_activities: None,
            segments: None,
            files: Vec::new(),
            timestamp: chrono::Utc::now(),
            tokens: 0,
            work_duration_ms: None,
            skill_names: None,
            skill_ids: None,
            stream_run_id: None,
            stream_part: None,
        }
    }
}
