use zeroize::Zeroize;

use super::{errors::VoiceError, limits};

pub struct Delivery {
    pub id: String,
    pub operation_id: String,
    pub draft_key: String,
    pub text: String,
    pub created_at_ms: u64,
    pub capture_ms: u64,
    pub speech_ms: u64,
}

pub struct TrialResult {
    pub trial_id: String,
    pub text: String,
    pub capture_ms: u64,
    pub compute_ms: u64,
}

impl Drop for TrialResult {
    fn drop(&mut self) {
        self.text.zeroize();
    }
}

impl Delivery {
    pub fn new(
        operation_id: String,
        draft_key: String,
        text: String,
        now_ms: u64,
        capture_ms: u64,
        speech_ms: u64,
    ) -> Result<Self, VoiceError> {
        if text.chars().count() > limits::MAX_TRANSCRIPT_CHARS {
            return Err(VoiceError::configuration_unavailable());
        }
        Ok(Self {
            id: uuid::Uuid::new_v4().to_string(),
            operation_id,
            draft_key,
            text,
            created_at_ms: now_ms,
            capture_ms,
            speech_ms,
        })
    }

    pub fn is_expired_at(&self, now_ms: u64) -> bool {
        now_ms.saturating_sub(self.created_at_ms) >= limits::DELIVERY_TIMEOUT_MS
    }
}

impl Drop for Delivery {
    fn drop(&mut self) {
        self.text.zeroize();
    }
}

use super::{
    actions::VoiceCoordinator,
    contracts::{VoiceDeliveryOutcome, VoiceDestination, VoiceSnapshot},
    recovery::{Recovery, RecoveryStatus},
    start_guards::validate_id,
};

impl VoiceCoordinator {
    pub fn restore_recovery(
        &mut self,
        recovery_id: &str,
        draft_key: String,
        now_ms: u64,
    ) -> Result<VoiceSnapshot, VoiceError> {
        validate_id(&draft_key, super::limits::MAX_DESTINATION_ID_CHARS)?;
        if self.delivery.is_some() {
            return Err(VoiceError::busy());
        }
        let Some(recovery) = self.recovery.take() else {
            return Err(VoiceError::invalid_transition());
        };
        if recovery.id != recovery_id || recovery.status != RecoveryStatus::Ready {
            self.recovery = Some(recovery);
            return Err(VoiceError::invalid_transition());
        }
        self.delivery = Some(Delivery::new(
            recovery.id.clone(),
            draft_key,
            recovery.text.clone(),
            now_ms,
            recovery.capture_ms,
            super::limits::RECOVERY_SPEECH_THRESHOLD_MS,
        )?);
        self.bump();
        Ok(self.snapshot())
    }

    pub fn acknowledge(
        &mut self,
        result_id: &str,
        outcome: VoiceDeliveryOutcome,
        now_ms: u64,
    ) -> Result<VoiceSnapshot, VoiceError> {
        validate_id(result_id, super::limits::MAX_DESTINATION_ID_CHARS)?;
        let Some(delivery) = self.delivery.take() else {
            return Ok(self.snapshot());
        };
        if delivery.id != result_id {
            self.delivery = Some(delivery);
            return Ok(self.snapshot());
        }
        if outcome == VoiceDeliveryOutcome::Closed
            && delivery.speech_ms >= super::limits::RECOVERY_SPEECH_THRESHOLD_MS
        {
            let mut recovery = Recovery::preparing(
                delivery.operation_id.clone(),
                &VoiceDestination::Draft {
                    draft_key: delivery.draft_key.clone(),
                },
                delivery.capture_ms,
            );
            recovery.attachment = None;
            let text = delivery.text.clone();
            let _ = recovery.mark_ready(text, now_ms);
            self.recovery = Some(recovery);
        }
        self.bump();
        Ok(self.snapshot())
    }

    pub fn complete(
        &mut self,
        operation_id: &str,
        text: String,
        now_ms: u64,
    ) -> Result<VoiceSnapshot, VoiceError> {
        self.complete_with_metrics(operation_id, text, now_ms, 0)
    }

    pub fn complete_with_metrics(
        &mut self,
        operation_id: &str,
        text: String,
        now_ms: u64,
        compute_ms: u64,
    ) -> Result<VoiceSnapshot, VoiceError> {
        let operation = self
            .operation
            .take()
            .ok_or_else(VoiceError::invalid_transition)?;
        if operation.id != operation_id {
            self.operation = Some(operation);
            return Err(VoiceError::invalid_transition());
        }
        if text.chars().count() > super::limits::MAX_TRANSCRIPT_CHARS {
            self.error = Some(VoiceError::configuration_unavailable());
            if let Some(recovery) = self.recovery.as_mut() {
                recovery.status = RecoveryStatus::Failed;
            }
        } else if operation.cancelled {
            if !operation.recovery_deleted {
                if let Some(recovery) = self.recovery.as_mut() {
                    let _ = recovery.mark_ready(text, now_ms);
                }
            }
        } else {
            match operation.destination.clone() {
                VoiceDestination::Draft { draft_key } => {
                    self.delivery = Some(Delivery::new(
                        operation.id.clone(),
                        draft_key,
                        text,
                        now_ms,
                        operation.capture_ms,
                        operation.speech_ms,
                    )?);
                }
                VoiceDestination::Trial { trial_id } => {
                    self.trial_result = Some(TrialResult {
                        trial_id,
                        text,
                        capture_ms: operation.capture_ms,
                        compute_ms,
                    });
                }
            }
        }
        drop(operation);
        self.bump();
        Ok(self.snapshot())
    }

    pub(super) fn cancel_delivery(
        &mut self,
        operation_id: &str,
    ) -> Result<VoiceSnapshot, VoiceError> {
        validate_id(operation_id, super::limits::MAX_DESTINATION_ID_CHARS)?;
        let Some(delivery) = self.delivery.take() else {
            return Ok(self.snapshot());
        };
        if delivery.operation_id != operation_id && delivery.id != operation_id {
            self.delivery = Some(delivery);
            return Ok(self.snapshot());
        }
        if delivery.speech_ms >= super::limits::RECOVERY_SPEECH_THRESHOLD_MS {
            let mut recovery = Recovery::preparing(
                uuid::Uuid::new_v4().to_string(),
                &VoiceDestination::Draft {
                    draft_key: delivery.draft_key.clone(),
                },
                delivery.capture_ms,
            );
            let _ = recovery.mark_ready(delivery.text.clone(), self.now_ms());
            self.recovery = Some(recovery);
        }
        self.bump();
        Ok(self.snapshot())
    }
}
