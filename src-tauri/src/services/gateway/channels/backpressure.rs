use super::InboundMessage;
use crate::services::gateway::refusal_audit::RefusalAudit;
use crate::services::gateway::types::ChannelKey;
use tokio::sync::mpsc;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum EnqueueOutcome {
    Enqueued,
    Full,
    Closed,
}

pub(super) fn try_enqueue(
    sender: &mpsc::Sender<InboundMessage>,
    message: InboundMessage,
    key: &ChannelKey,
    audit: &RefusalAudit,
) -> EnqueueOutcome {
    match sender.try_send(message) {
        Ok(()) => EnqueueOutcome::Enqueued,
        Err(mpsc::error::TrySendError::Full(_)) => {
            audit.try_record(key.clone(), "gateway_busy");
            EnqueueOutcome::Full
        }
        Err(mpsc::error::TrySendError::Closed(_)) => {
            audit.try_record(key.clone(), "gateway_shutting_down");
            EnqueueOutcome::Closed
        }
    }
}
