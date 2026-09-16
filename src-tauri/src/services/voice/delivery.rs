use super::{
    actions::VoiceCoordinator,
    contracts::{VoiceDeliveryOutcome, VoiceDestination, VoiceSnapshot},
    delivery_types::{Delivery, TrialResult},
    errors::VoiceError,
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
        if self.operation.is_some() {
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
        ::log::info!("[voice] operation={} step=recovery-restored", recovery.id);
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
        ::log::info!(
            "[voice] operation={} step=delivery-acknowledged outcome={outcome:?}",
            delivery.operation_id
        );
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

    #[cfg(test)]
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
        let text_chars = text.chars().count();
        let capture_ms = operation.capture_ms;
        let lost_samples = operation.lost_samples;
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
                    let mut delivery = Delivery::new(
                        operation.id.clone(),
                        draft_key,
                        text,
                        now_ms,
                        operation.capture_ms,
                        operation.speech_ms,
                    )?;
                    delivery.microphone_disconnected = operation.microphone_disconnected;
                    self.delivery = Some(delivery);
                }
                VoiceDestination::Trial { trial_id } => {
                    self.trial_result = Some(TrialResult {
                        trial_id,
                        text,
                        capture_ms: operation.capture_ms,
                        compute_ms,
                        created_at_ms: now_ms,
                    });
                }
            }
        }
        drop(operation);
        ::log::info!(
            "[voice] operation={} step=completed capture_ms={} compute_ms={} text_chars={} lost_samples={}",
            operation_id,
            capture_ms,
            compute_ms,
            text_chars,
            lost_samples
        );
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
