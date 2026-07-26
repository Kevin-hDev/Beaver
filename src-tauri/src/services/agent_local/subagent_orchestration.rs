use super::stream_events::AgentEventEmitter;
use super::types_ollama::ChatMessage;
use super::types_stream::{StreamEvent, StreamResult};
use std::collections::BTreeSet;
use std::sync::Arc;
use tokio_util::sync::CancellationToken;

pub struct ParentSubagentOrchestrator {
    parent_session_id: String,
    reports_injected_since_last_request: bool,
    report_delivery: super::subagent_report_delivery::SubagentReportDelivery,
    parent_message_inbox: Option<Arc<super::parent_message_inbox::ParentMessageInbox>>,
}

impl ParentSubagentOrchestrator {
    #[cfg(test)]
    pub async fn new(parent_session_id: &str) -> Self {
        Self::with_parent_inbox(parent_session_id, None).await
    }

    pub async fn with_parent_inbox(
        parent_session_id: &str,
        parent_message_inbox: Option<Arc<super::parent_message_inbox::ParentMessageInbox>>,
    ) -> Self {
        Self {
            parent_session_id: parent_session_id.to_string(),
            reports_injected_since_last_request: false,
            report_delivery: super::subagent_report_delivery::SubagentReportDelivery::new(
                parent_session_id,
            ),
            parent_message_inbox,
        }
    }

    pub async fn inject_pending_reports(&mut self, messages: &mut Vec<ChatMessage>) -> bool {
        self.report_delivery.inject_pending_reports(messages).await
    }

    pub async fn complete_model_request(
        &mut self,
        successful: bool,
        cancel: &CancellationToken,
        payload: &[ChatMessage],
    ) -> Result<(), String> {
        self.report_delivery
            .complete_model_request(successful, cancel, payload)
            .await
    }

    pub async fn prepare_for_model_request(
        &mut self,
        messages: &mut Vec<ChatMessage>,
    ) -> Result<(), String> {
        super::subagent_instruction_delivery::drain(&self.parent_session_id, messages).await?;
        self.report_delivery.refresh_terminal_signal().await;
        if self.report_delivery.persistence_failed() {
            return Err(super::subagent_completion::SUBAGENT_COMPLETION_ERROR.to_string());
        }
        super::subagent_orchestration_context::remove_gate_context(messages);
        let reports_injected =
            self.reports_injected_since_last_request || self.inject_pending_reports(messages).await;
        self.reports_injected_since_last_request = false;
        let active_ids = self.current_turn_active_ids().await;
        super::subagent_orchestration_context::replace_gate_context(
            messages,
            active_ids.len(),
            reports_injected,
        );
        Ok(())
    }

    pub async fn finalize_content_phase(
        &self,
        on_event: &AgentEventEmitter,
        result: &StreamResult,
        plan_active: bool,
    ) {
        let awaiting_report = super::subagent_hidden_reports::has_pending_except(
            &self.parent_session_id,
            &BTreeSet::new(),
        )
        .await;
        let terminal_failure = self.report_delivery.persistence_failed();
        super::stream_buffer::finalize_content_phase(
            on_event,
            result,
            plan_active,
            awaiting_report || terminal_failure || !self.current_turn_active_ids().await.is_empty(),
        );
    }
    pub async fn continue_after_no_tool_turn(
        &mut self,
        on_event: &AgentEventEmitter,
        messages: &mut Vec<ChatMessage>,
        cancel: CancellationToken,
        can_request_again: bool,
    ) -> Result<bool, String> {
        if !can_request_again {
            self.ensure_no_followup_at_turn_limit().await?;
            return Ok(false);
        }
        let should_continue = self.drain_parent_messages(messages).await > 0
            || super::subagent_instruction_delivery::drain(
            &self.parent_session_id,
            messages,
        )
        .await?
            > 0
            || self.after_no_tool_turn(messages, cancel).await?;
        if should_continue {
            let _ = on_event.send(StreamEvent::TurnEnd {});
        }
        Ok(should_continue)
    }

    pub async fn ensure_no_followup_at_turn_limit(&mut self) -> Result<(), String> {
        if let Some(inbox) = &self.parent_message_inbox {
            inbox.close().await;
        }
        super::subagent_turn_limit::ensure(&self.parent_session_id, &mut self.report_delivery).await
    }

    pub async fn wait_after_tool_batch(
        &mut self,
        control_only: bool,
        messages: &mut Vec<ChatMessage>,
        cancel: CancellationToken,
    ) -> Result<(), String> {
        if self.drain_parent_messages(messages).await > 0 {
            return Ok(());
        }
        if control_only {
            let _ = self.after_no_tool_turn(messages, cancel).await?;
        }
        Ok(())
    }

    async fn current_turn_active_ids(&self) -> Vec<String> {
        current_turn_active_ids(&self.parent_session_id).await
    }

    async fn drain_parent_messages(&self, messages: &mut Vec<ChatMessage>) -> usize {
        match &self.parent_message_inbox {
            Some(inbox) => inbox.drain_into(messages).await,
            None => 0,
        }
    }

    async fn finish_parent_messages(&self, messages: &mut Vec<ChatMessage>) -> bool {
        match &self.parent_message_inbox {
            Some(inbox) => inbox.finish_or_drain(messages).await,
            None => false,
        }
    }
}

#[path = "subagent_orchestration_wait.rs"]
mod wait;

async fn current_turn_active_ids(parent_session_id: &str) -> Vec<String> {
    super::subagent_registry::active_children_for_parent(parent_session_id).await
}

#[cfg(test)]
#[path = "subagent_orchestration_tests.rs"]
mod tests;
