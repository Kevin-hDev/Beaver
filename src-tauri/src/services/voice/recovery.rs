use zeroize::Zeroize;

use super::{contracts::VoiceDestination, limits};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RecoveryStatus {
    Preparing,
    Ready,
    Failed,
}

pub struct Recovery {
    pub id: String,
    pub attachment: Option<String>,
    pub capture_ms: u64,
    pub status: RecoveryStatus,
    pub ready_at_ms: Option<u64>,
    pub text: String,
}

impl Recovery {
    pub fn preparing(id: String, destination: &VoiceDestination, capture_ms: u64) -> Self {
        Self {
            id,
            attachment: match destination {
                VoiceDestination::Draft { draft_key } => Some(draft_key.clone()),
                VoiceDestination::Trial { .. } => None,
            },
            capture_ms,
            status: RecoveryStatus::Preparing,
            ready_at_ms: None,
            text: String::new(),
        }
    }

    pub fn mark_ready(&mut self, text: String, now_ms: u64) -> bool {
        if text.chars().count() > limits::MAX_TRANSCRIPT_CHARS {
            return false;
        }
        self.text.zeroize();
        self.text = text;
        self.status = RecoveryStatus::Ready;
        self.ready_at_ms = Some(now_ms);
        true
    }

    pub fn is_expired_at(&self, now_ms: u64) -> bool {
        self.ready_at_ms
            .is_some_and(|ready| now_ms.saturating_sub(ready) >= limits::RECOVERY_TTL_MS)
    }
}

impl Drop for Recovery {
    fn drop(&mut self) {
        self.text.zeroize();
    }
}

use super::{
    actions::VoiceCoordinator,
    contracts::VoiceSnapshot,
    errors::VoiceError,
    start_guards::validate_id,
    state::{cancel_disposition, CancelDisposition},
};

impl VoiceCoordinator {
    pub fn disable(&mut self) -> VoiceSnapshot {
        if let Some(operation) = self.operation.as_mut() {
            operation.cancelled = true;
            operation.recovery_deleted = true;
            let _ = operation
                .reservation
                .context()
                .transition(super::types::VoicePhase::Stopping);
        }
        self.recovery.take();
        self.delivery.take();
        self.trial_result.take();
        self.bump();
        self.snapshot()
    }

    pub fn cancel_insertion(&mut self, operation_id: &str) -> Result<VoiceSnapshot, VoiceError> {
        let Some(operation) = self.operation.as_mut() else {
            return self.cancel_delivery(operation_id);
        };
        if operation.id != operation_id || operation.cancelled {
            return Ok(self.snapshot());
        }
        let recovers = cancel_disposition(operation.speech_ms) == CancelDisposition::Recover
            && matches!(operation.destination, VoiceDestination::Draft { .. });
        operation.reservation.context().transition(if recovers {
            super::types::VoicePhase::Recovering
        } else {
            super::types::VoicePhase::Stopping
        })?;
        operation.cancelled = true;
        if recovers {
            self.recovery = Some(Recovery::preparing(
                uuid::Uuid::new_v4().to_string(),
                &operation.destination,
                operation.capture_ms,
            ));
        }
        self.bump();
        Ok(self.snapshot())
    }

    pub fn abandon_trial(&mut self, trial_id: &str) -> Result<VoiceSnapshot, VoiceError> {
        validate_id(trial_id, super::limits::MAX_DESTINATION_ID_CHARS)?;
        if let Some(operation) = self.operation.as_mut() {
            if matches!(&operation.destination, VoiceDestination::Trial { trial_id: current } if current == trial_id)
            {
                operation.cancelled = true;
                operation.recovery_deleted = true;
                let _ = operation
                    .reservation
                    .context()
                    .transition(super::types::VoicePhase::Stopping);
                self.recovery.take();
                self.bump();
            }
        }
        if self
            .trial_result
            .as_ref()
            .is_some_and(|result| result.trial_id == trial_id)
        {
            self.trial_result.take();
            self.bump();
        }
        Ok(self.snapshot())
    }

    pub fn delete_recovery(&mut self, recovery_id: &str) -> Result<VoiceSnapshot, VoiceError> {
        validate_id(recovery_id, super::limits::MAX_DESTINATION_ID_CHARS)?;
        if self
            .recovery
            .as_ref()
            .is_some_and(|item| item.id == recovery_id)
        {
            self.recovery.take();
            if let Some(operation) = self.operation.as_mut() {
                operation.recovery_deleted = true;
                let _ = operation
                    .reservation
                    .context()
                    .transition(super::types::VoicePhase::Stopping);
            }
            self.bump();
        }
        Ok(self.snapshot())
    }

    pub fn destination_closed(
        &mut self,
        destination: &VoiceDestination,
    ) -> Result<VoiceSnapshot, VoiceError> {
        if let VoiceDestination::Trial { trial_id } = destination {
            return self.abandon_trial(trial_id);
        }
        if let VoiceDestination::Draft { draft_key } = destination {
            if let Some(recovery) = self.recovery.as_mut() {
                if recovery.attachment.as_deref() == Some(draft_key) {
                    recovery.attachment = None;
                    self.bump();
                }
            }
            if let Some(operation) = self.operation.as_ref() {
                if operation.destination == *destination {
                    let id = operation.id.clone();
                    self.cancel_insertion(&id)?;
                    if let Some(recovery) = self.recovery.as_mut() {
                        recovery.attachment = None;
                    }
                    return Ok(self.snapshot());
                }
            }
            if let Some(delivery) = self.delivery.as_ref() {
                if delivery.draft_key == *draft_key {
                    let operation_id = delivery.operation_id.clone();
                    self.cancel_delivery(&operation_id)?;
                    if let Some(recovery) = self.recovery.as_mut() {
                        recovery.attachment = None;
                    }
                    return Ok(self.snapshot());
                }
            }
        }
        Ok(self.snapshot())
    }

    pub fn message_accepted(
        &mut self,
        draft_key: &str,
        send_id: &str,
    ) -> Result<VoiceSnapshot, VoiceError> {
        validate_id(draft_key, super::limits::MAX_DESTINATION_ID_CHARS)?;
        validate_id(send_id, super::limits::MAX_SEND_ID_CHARS)?;
        if self
            .recovery
            .as_ref()
            .is_some_and(|item| item.attachment.as_deref() == Some(draft_key))
        {
            self.recovery.take();
            if let Some(operation) = self.operation.as_mut() {
                operation.recovery_deleted = true;
                let _ = operation
                    .reservation
                    .context()
                    .transition(super::types::VoicePhase::Stopping);
            }
            self.bump();
        }
        Ok(self.snapshot())
    }
}
