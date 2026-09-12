use super::stream_recovery_record::{
    RecoverableStreamEvent, RecoverableToolResult, StreamRecoveryHeader, StreamRecoveryRecord,
};
use super::types_message::{AgentMessage, SavedSegment, ToolActivityRecord, ToolCallRequest};
use super::types_stream::TokenPhase;

pub(crate) struct RecoveryProjection {
    pub(crate) header: StreamRecoveryHeader,
    pub(crate) messages: Vec<AgentMessage>,
    pub(crate) turn_ready: bool,
}

#[derive(Default)]
pub(super) struct EventProjection {
    pub(super) content: String,
    pub(super) thinking: String,
    pub(super) phase: Option<TokenPhase>,
    pub(super) calls: Vec<ToolCallRequest>,
    pub(super) activities: Vec<ToolActivityRecord>,
    pub(super) results: Vec<RecoverableToolResult>,
    pub(super) segments: Vec<SavedSegment>,
}

pub(crate) fn from_path(path: &std::path::Path) -> Result<RecoveryProjection, String> {
    let mut header = None;
    let mut events = EventProjection::default();
    let mut pending = Vec::new();
    let mut pending_identity = None;
    let mut turn_ready = false;
    super::stream_recovery_store::visit_records(path, |record| {
        match record {
            StreamRecoveryRecord::Header(value) => header = Some(value),
            StreamRecoveryRecord::Event { event, .. } => events.apply(event)?,
            StreamRecoveryRecord::PendingMessage {
                batch_id,
                position,
                total,
                message,
                ..
            } => {
                if pending_identity
                    .as_ref()
                    .is_some_and(|identity| identity != &(batch_id.clone(), total))
                    || position != pending.len()
                    || total == 0
                    || total > super::session_limits::MAX_MESSAGES_PER_SESSION
                {
                    return Err(error());
                }
                pending_identity.get_or_insert((batch_id, total));
                pending.push(message);
            }
            StreamRecoveryRecord::TurnReady { .. } => turn_ready = true,
        }
        Ok(())
    })?;
    let header = header.ok_or_else(error)?;
    let mut messages = if pending.is_empty() {
        super::stream_recovery_projection_messages::finish(events, &header)?
    } else {
        let expected = pending_identity.map(|(_, total)| total).ok_or_else(error)?;
        if pending.len() != expected {
            return Err(error());
        }
        pending
    };
    if turn_ready {
        for message in &mut messages {
            if message.stream_run_id.as_deref() == Some(&header.request_id) {
                message.stream_part = Some("final".into());
            }
        }
    }
    for message in &messages {
        validate_message(message, &header)?;
    }
    Ok(RecoveryProjection {
        header,
        messages,
        turn_ready,
    })
}

fn validate_message(message: &AgentMessage, header: &StreamRecoveryHeader) -> Result<(), String> {
    if message.turn_id != header.turn_id
        || message.stream_run_id.as_deref() != Some(&header.request_id)
        || !matches!(message.role.as_str(), "assistant" | "tool")
        || super::session_migration_ids::validate_id(&message.id).is_err()
        || super::session_migration_ids::validate_id(&message.turn_id).is_err()
        || message.validate_stream_metadata().is_err()
        || super::conversation_history_field_validation::validate(message).is_err()
    {
        return Err(error());
    }
    Ok(())
}

impl EventProjection {
    fn apply(&mut self, event: RecoverableStreamEvent) -> Result<(), String> {
        match event {
            RecoverableStreamEvent::Token { content, phase } => {
                if let Some(phase) = phase {
                    self.change_phase(phase)?;
                }
                self.content.push_str(&content);
            }
            RecoverableStreamEvent::Thinking { content } => self.thinking.push_str(&content),
            RecoverableStreamEvent::ContentPhase { phase } => self.change_phase(phase)?,
            RecoverableStreamEvent::AttemptRestarted { .. } => *self = Self::default(),
            RecoverableStreamEvent::ToolCall(call) => {
                if call.tool_call_index != self.calls.len()
                    || self.calls.len()
                        >= crate::services::reasoning_continuity::limits::MAX_TOOL_CALLS
                {
                    return Err(error());
                }
                self.activities.push(
                    super::stream_recovery_projection_messages::activity_for_call(
                        &call.name,
                        call.arguments.clone(),
                        call.domain,
                    ),
                );
                self.calls.push(super::types_message::ToolCallRequest {
                    id: call.tool_call_id.ok_or_else(error)?,
                    extra_content: call.extra_content,
                    function: super::types_message::ToolCallRequestFunction {
                        name: call.name,
                        arguments: call.arguments,
                    },
                });
            }
            RecoverableStreamEvent::ToolResult(result) => {
                if result.tool_call_index
                    >= crate::services::reasoning_continuity::limits::MAX_TOOL_CALLS
                    || uuid::Uuid::parse_str(&result.message_id).is_err()
                    || self.results.len()
                        >= crate::services::reasoning_continuity::limits::MAX_TOOL_CALLS
                    || self.results.iter().any(|existing| {
                    existing.tool_call_index == result.tool_call_index
                        || (existing.tool_call_id.is_some()
                            && existing.tool_call_id == result.tool_call_id)
                }) {
                    return Err(error());
                }
                let matches = self.calls.get(result.tool_call_index).is_some_and(|call| {
                    result
                        .tool_call_id
                        .as_deref()
                        .map_or(call.function.name == result.name, |id| call.id == id)
                });
                if !self.calls.is_empty() && !matches {
                    return Err(error());
                }
                if let Some(activity) = self.activities.get_mut(result.tool_call_index) {
                    super::stream_recovery_projection_messages::apply_result(activity, &result);
                }
                self.results.push(result);
            }
        }
        Ok(())
    }

    fn change_phase(&mut self, phase: TokenPhase) -> Result<(), String> {
        if self
            .phase
            .as_ref()
            .is_some_and(|current| !same_phase(current, &phase))
            && self.has_visible_data()
        {
            self.push_segment()?;
        }
        self.phase = Some(phase);
        Ok(())
    }

    fn has_visible_data(&self) -> bool {
        !self.content.is_empty() || !self.thinking.is_empty() || !self.activities.is_empty()
    }

    fn push_segment(&mut self) -> Result<(), String> {
        if self.segments.len() >= super::session_limits::MAX_MESSAGES_PER_SESSION {
            return Err(error());
        }
        self.segments.push(SavedSegment {
            thinking: (!self.thinking.is_empty()).then(|| std::mem::take(&mut self.thinking)),
            tools: std::mem::take(&mut self.activities),
            content: std::mem::take(&mut self.content),
            phase: self.phase.take(),
        });
        Ok(())
    }
}

fn same_phase(left: &TokenPhase, right: &TokenPhase) -> bool {
    matches!(
        (left, right),
        (TokenPhase::Work, TokenPhase::Work) | (TokenPhase::Final, TokenPhase::Final)
    )
}

fn error() -> String {
    "stream_recovery_invalid".into()
}
