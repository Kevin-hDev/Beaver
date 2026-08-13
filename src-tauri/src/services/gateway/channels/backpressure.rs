use super::InboundMessage;
use crate::services::gateway::refusal_audit::RefusalAudit;
use crate::services::gateway::types::ChannelKey;
use tokio::sync::mpsc;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[must_use = "Closed must stop the channel consumer"]
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
            let _ = audit.record_refusal(key.clone(), "gateway_busy");
            EnqueueOutcome::Full
        }
        Err(mpsc::error::TrySendError::Closed(_)) => {
            let _ = audit.record_refusal(key.clone(), "gateway_shutting_down");
            EnqueueOutcome::Closed
        }
    }
}
