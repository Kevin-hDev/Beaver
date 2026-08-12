use super::InboundMessage;
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
) -> EnqueueOutcome {
    match sender.try_send(message) {
        Ok(()) => EnqueueOutcome::Enqueued,
        Err(mpsc::error::TrySendError::Full(_)) => {
            record_refusal(key, "gateway_busy");
            EnqueueOutcome::Full
        }
        Err(mpsc::error::TrySendError::Closed(_)) => {
            record_refusal(key, "gateway_shutting_down");
            EnqueueOutcome::Closed
        }
    }
}

fn record_refusal(key: &ChannelKey, decision: &str) {
    if crate::services::gateway::service_audit::work_refused(key, decision).is_err() {
        ::log::warn!("[gateway] audit indisponible pour un refus de file");
    }
}
