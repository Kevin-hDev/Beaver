use super::ParentSubagentOrchestrator;
use crate::services::agent_local::{subagent_completion, subagent_registry};
use crate::services::agent_local::types_ollama::ChatMessage;
use tokio_util::sync::CancellationToken;

impl ParentSubagentOrchestrator {
    pub async fn after_no_tool_turn(
        &mut self,
        messages: &mut Vec<ChatMessage>,
        cancel: CancellationToken,
    ) -> Result<bool, String> {
        loop {
            if cancel.is_cancelled() {
                return Err("Annulé".to_string());
            }
            let mut terminal_signal =
                subagent_registry::subscribe_for_parent(&self.parent_session_id).await;
            self.report_delivery.refresh_terminal_signal().await;
            if self.report_delivery.persistence_failed() {
                return Err(subagent_completion::SUBAGENT_COMPLETION_ERROR.to_string());
            }
            if self.inject_pending_reports(messages).await {
                self.reports_injected_since_last_request = true;
                return Ok(true);
            }
            if self.drain_parent_messages(messages).await > 0 {
                return Ok(true);
            }
            let snapshot = subagent_registry::parent_snapshot(&self.parent_session_id).await;
            if snapshot.active_child_ids.is_empty() {
                if terminal_failed(&snapshot) {
                    return Err(subagent_completion::SUBAGENT_COMPLETION_ERROR.to_string());
                }
                if terminal_completed(&snapshot) {
                    if self.inject_pending_reports(messages).await {
                        self.reports_injected_since_last_request = true;
                        return Ok(true);
                    }
                    return Err(subagent_completion::SUBAGENT_COMPLETION_ERROR.to_string());
                }
                return Ok(self.finish_parent_messages(messages).await);
            }
            let signal = terminal_signal
                .as_mut()
                .ok_or_else(|| subagent_completion::SUBAGENT_COMPLETION_ERROR.to_string())?;
            if let Some(inbox) = &self.parent_message_inbox {
                let mut input = inbox.subscribe();
                tokio::select! {
                    _ = cancel.cancelled() => return Err("Annulé".to_string()),
                    changed = signal.changed() => changed.map_err(|_| generic_error())?,
                    changed = input.changed() => changed.map_err(|_| generic_input_error())?,
                }
            } else {
                tokio::select! {
                    _ = cancel.cancelled() => return Err("Annulé".to_string()),
                    changed = signal.changed() => changed.map_err(|_| generic_error())?,
                }
            }
        }
    }
}

fn terminal_failed(snapshot: &subagent_registry::ParentRegistrySnapshot) -> bool {
    snapshot
        .terminal_state
        .is_some_and(|state| state.report_persistence_failed)
}

fn terminal_completed(snapshot: &subagent_registry::ParentRegistrySnapshot) -> bool {
    snapshot
        .terminal_state
        .is_some_and(|state| state.sequence > 0)
}

fn generic_error() -> String {
    subagent_completion::SUBAGENT_COMPLETION_ERROR.to_string()
}

fn generic_input_error() -> String {
    "Impossible de recevoir ce message".to_string()
}
