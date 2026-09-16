use super::{
    contracts::{
        VoiceDeliverySnapshot, VoiceDestination, VoiceOperationSnapshot, VoiceRecoverySnapshot,
        VoiceRecoveryState, VoiceSnapshot, VoiceTrialResult,
    },
    delivery_types::{Delivery, TrialResult},
    errors::VoiceError,
    operation::Operation,
    recovery::{Recovery, RecoveryStatus},
    runtime::VoiceReservation,
    start_guards::{validate_id, validate_start},
    types::{VoiceLanguage, VoicePhase, VoiceSettings},
};

pub struct VoiceCoordinator {
    pub(super) revision: u64,
    pub(super) operation: Option<Operation>,
    pub(super) recovery: Option<Recovery>,
    pub(super) delivery: Option<Delivery>,
    pub(super) trial_result: Option<TrialResult>,
    pub(super) error: Option<VoiceError>,
    clock: std::time::Instant,
}

impl Default for VoiceCoordinator {
    fn default() -> Self {
        Self {
            revision: 0,
            operation: None,
            recovery: None,
            delivery: None,
            trial_result: None,
            error: None,
            clock: std::time::Instant::now(),
        }
    }
}

impl VoiceCoordinator {
    pub fn start(
        &mut self,
        reservation: VoiceReservation,
        destination: VoiceDestination,
        context_generation: u64,
        language: Option<VoiceLanguage>,
        settings: VoiceSettings,
        foreground: bool,
    ) -> Result<VoiceSnapshot, VoiceError> {
        validate_start(&destination, context_generation, &language, foreground)?;
        if self.operation.is_some() || self.delivery.is_some() {
            return Err(VoiceError::busy());
        }
        self.error.take();
        let operation_id = uuid::Uuid::new_v4().to_string();
        self.operation = Some(Operation {
            id: operation_id.clone(),
            destination,
            context_generation,
            capture_ms: 0,
            speech_ms: 0,
            lost_samples: 0,
            microphone_disconnected: false,
            level: 0.0,
            cancelled: false,
            recovery_deleted: false,
            reservation,
        });
        ::log::info!(
            "[voice] operation={} step=started model={:?} language={:?}",
            operation_id,
            settings.model,
            language
        );
        self.bump();
        Ok(self.snapshot())
    }

    pub fn validate(&mut self, operation_id: &str) -> Result<VoiceSnapshot, VoiceError> {
        let operation = self.matching_operation_mut(operation_id)?;
        operation
            .reservation
            .context()
            .transition(VoicePhase::Transcribing)?;
        ::log::info!("[voice] operation={operation_id} step=validation-requested");
        self.bump();
        Ok(self.snapshot())
    }

    pub fn record_capture(
        &mut self,
        operation_id: &str,
        capture_ms: u64,
        speech_ms: u64,
        lost_samples: u64,
        level: f32,
    ) -> Result<(), VoiceError> {
        let operation = self.matching_operation_mut(operation_id)?;
        operation.capture_ms = capture_ms;
        operation.speech_ms = speech_ms.min(capture_ms);
        operation.lost_samples = lost_samples;
        operation.level = level.clamp(0.0, 1.0);
        self.bump();
        Ok(())
    }

    pub fn begin_listening(&mut self, operation_id: &str) -> Result<(), VoiceError> {
        self.matching_operation_mut(operation_id)?
            .reservation
            .context()
            .transition(VoicePhase::Listening)?;
        ::log::info!("[voice] operation={operation_id} step=listening");
        self.bump();
        Ok(())
    }

    pub fn stop_without_result(&mut self, operation_id: &str) -> Result<(), VoiceError> {
        let operation = self
            .operation
            .take()
            .ok_or_else(VoiceError::invalid_transition)?;
        if operation.id != operation_id {
            self.operation = Some(operation);
            return Err(VoiceError::invalid_transition());
        }
        drop(operation);
        ::log::info!("[voice] operation={operation_id} step=stopped-without-result");
        self.bump();
        Ok(())
    }

    pub fn fail(&mut self, operation_id: &str, error: VoiceError) {
        if self
            .operation
            .as_ref()
            .is_some_and(|item| item.id == operation_id)
        {
            self.operation.take();
            if let Some(recovery) = self.recovery.as_mut() {
                if recovery.status == RecoveryStatus::Preparing {
                    recovery.status = RecoveryStatus::Failed;
                }
            }
            self.error = Some(error);
            ::log::warn!(
                "[voice] operation={} step=failed code={:?}",
                operation_id,
                self.error.as_ref().map(VoiceError::code)
            );
            self.bump();
        }
    }

    pub fn clear_error(&mut self) -> VoiceSnapshot {
        if self.error.take().is_some() {
            self.bump();
        }
        self.snapshot()
    }

    pub fn snapshot(&self) -> VoiceSnapshot {
        let phase = if self.delivery.is_some() {
            VoicePhase::Delivering
        } else if let Some(operation) = self.operation.as_ref() {
            operation.reservation.context().phase()
        } else {
            VoicePhase::Idle
        };
        VoiceSnapshot {
            revision: self.revision,
            phase,
            operation: self.operation.as_ref().map(|item| VoiceOperationSnapshot {
                id: item.id.clone(),
                destination: item.destination.clone(),
                context_generation: item.context_generation,
                capture_ms: item.capture_ms,
                speech_ms: item.speech_ms,
                capture_incomplete: item.lost_samples > 0,
                level: item.level,
            }),
            recovery: self.recovery.as_ref().map(|item| VoiceRecoverySnapshot {
                id: item.id.clone(),
                draft_key: item.attachment.clone(),
                capture_ms: item.capture_ms,
                status: match item.status {
                    RecoveryStatus::Preparing => VoiceRecoveryState::Preparing,
                    RecoveryStatus::Ready => VoiceRecoveryState::Ready,
                    RecoveryStatus::Failed => VoiceRecoveryState::Failed,
                },
            }),
            delivery: self.delivery.as_ref().map(|item| VoiceDeliverySnapshot {
                id: item.id.clone(),
                draft_key: item.draft_key.clone(),
                text: item.text.clone(),
                microphone_disconnected: item.microphone_disconnected,
            }),
            trial_result: self.trial_result.as_ref().map(|item| VoiceTrialResult {
                trial_id: item.trial_id.clone(),
                text: item.text.clone(),
                capture_ms: item.capture_ms,
                compute_ms: item.compute_ms,
            }),
            error: self.error.clone(),
        }
    }

    fn matching_operation_mut(&mut self, id: &str) -> Result<&mut Operation, VoiceError> {
        validate_id(id, super::limits::MAX_DESTINATION_ID_CHARS)?;
        self.operation
            .as_mut()
            .filter(|item| item.id == id)
            .ok_or_else(VoiceError::invalid_transition)
    }

    pub fn now_ms(&self) -> u64 {
        u64::try_from(self.clock.elapsed().as_millis()).unwrap_or(u64::MAX)
    }

    pub(super) fn bump(&mut self) {
        self.revision = self.revision.wrapping_add(1).max(1);
    }
}
