use super::subagent_terminal_signal::SubagentTerminalState;
use super::types_ollama::ChatMessage;
use std::collections::BTreeSet;

const REPORT_ACK_ERROR: &str = "Impossible de finaliser les rapports de sous-agents.";

pub struct SubagentReportDelivery {
    parent_session_id: String,
    pending_report_ids: BTreeSet<String>,
    pending_report_payloads: Vec<String>,
    terminal_signal: Option<tokio::sync::watch::Receiver<SubagentTerminalState>>,
}

impl SubagentReportDelivery {
    pub fn new(parent_session_id: &str) -> Self {
        Self {
            parent_session_id: parent_session_id.to_string(),
            pending_report_ids: BTreeSet::new(),
            pending_report_payloads: Vec::new(),
            terminal_signal: None,
        }
    }

    pub async fn inject_pending_reports(&mut self, messages: &mut Vec<ChatMessage>) -> bool {
        let available = super::subagent_hidden_reports::peek_reports(&self.parent_session_id).await;
        self.pending_report_ids
            .retain(|id| available.iter().any(|report| report.id == id.as_str()));
        let reports = available
            .into_iter()
            .filter(|report| !self.pending_report_ids.contains(&report.id))
            .collect::<Vec<_>>();
        if reports.is_empty() {
            return false;
        }
        self.pending_report_ids
            .extend(reports.iter().map(|report| report.id.clone()));
        let previous_len = messages.len();
        super::subagent_hidden_reports::append_context(messages, &reports);
        self.pending_report_payloads.extend(
            messages[previous_len..]
                .iter()
                .filter(|message| {
                    message
                        .content
                        .starts_with(super::subagent_report_context::SUBAGENT_REPORT_CONTEXT_PREFIX)
                })
                .map(|message| message.content.clone()),
        );
        true
    }

    pub async fn refresh_terminal_signal(&mut self) {
        let current = super::subagent_registry::terminal_state_for_parent(&self.parent_session_id)
            .await;
        let subscribed_generation = self
            .terminal_signal
            .as_ref()
            .map(|receiver| receiver.borrow().generation);
        if current.map(|state| state.generation) != subscribed_generation {
            self.terminal_signal =
                super::subagent_registry::subscribe_for_parent(&self.parent_session_id).await;
        }
    }

    pub fn persistence_failed(&self) -> bool {
        self.terminal_signal
            .as_ref()
            .is_some_and(|receiver| receiver.borrow().report_persistence_failed)
    }

    pub async fn complete_model_request(
        &mut self,
        successful: bool,
        cancel: &tokio_util::sync::CancellationToken,
        payload: &[ChatMessage],
    ) -> Result<(), String> {
        self.refresh_terminal_signal().await;
        if self.persistence_failed() {
            return Err(super::subagent_completion::SUBAGENT_COMPLETION_ERROR.to_string());
        }
        if !successful || self.pending_report_ids.is_empty() {
            return Ok(());
        }
        if cancel.is_cancelled() {
            return Err("Annulé".to_string());
        }
        let all_reports_present = self.pending_report_payloads.iter().all(|expected| {
            payload
                .iter()
                .any(|message| message.content == expected.as_str())
        });
        if !all_reports_present {
            return Err(REPORT_ACK_ERROR.to_string());
        }
        let terminal_state = self
            .terminal_signal
            .as_ref()
            .map(|receiver| *receiver.borrow())
            .filter(|state| state.sequence > 0);
        let report_ids = self.pending_report_ids.iter().cloned().collect::<Vec<_>>();
        if cancel.is_cancelled() {
            return Err("Annulé".to_string());
        }
        super::subagent_hidden_reports::acknowledge_reports_cancellable(
            &self.parent_session_id,
            &report_ids,
            cancel,
        )
        .await
        .map_err(|_| REPORT_ACK_ERROR.to_string())?;
        self.pending_report_ids.clear();
        self.pending_report_payloads.clear();
        if let Some(state) = terminal_state {
            let consumed = super::subagent_registry::consume_terminal(
                &self.parent_session_id,
                state.generation,
                state.sequence,
            )
            .await;
            if consumed {
                self.terminal_signal = None;
            } else {
                self.refresh_terminal_signal().await;
                if self.persistence_failed() {
                    return Err(
                        super::subagent_completion::SUBAGENT_COMPLETION_ERROR.to_string(),
                    );
                }
            }
        }
        Ok(())
    }
}
