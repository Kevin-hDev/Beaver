use super::{
    contracts::{
        VoiceDeliverySnapshot, VoiceDestination, VoiceOperationSnapshot, VoiceRecoverySnapshot,
        VoiceRecoveryState, VoiceSnapshot, VoiceTrialResult,
    },
    delivery::{Delivery, TrialResult},
    errors::VoiceError,
    recovery::{Recovery, RecoveryStatus},
    runtime::VoiceReservation,
    start_guards::{validate_id, validate_start},
    types::{VoiceLanguage, VoicePhase, VoiceSettings},
};

pub(super) struct Operation {
    pub(super) id: String,
    pub(super) destination: VoiceDestination,
    pub(super) context_generation: u64,
    pub(super) language: Option<VoiceLanguage>,
    pub(super) settings: VoiceSettings,
    pub(super) capture_ms: u64,
    pub(super) speech_ms: u64,
    pub(super) capture_incomplete: bool,
    pub(super) cancelled: bool,
    pub(super) recovery_deleted: bool,
    pub(super) reservation: VoiceReservation,
}

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
        self.operation = Some(Operation {
            id: uuid::Uuid::new_v4().to_string(),
            destination,
            context_generation,
            language,
            settings,
            capture_ms: 0,
            speech_ms: 0,
            capture_incomplete: false,
            cancelled: false,
            recovery_deleted: false,
            reservation,
        });
        self.bump();
        Ok(self.snapshot())
    }

    pub fn validate(&mut self, operation_id: &str) -> Result<VoiceSnapshot, VoiceError> {
        let operation = self.matching_operation_mut(operation_id)?;
        operation
            .reservation
            .context()
            .transition(VoicePhase::Transcribing)?;
        self.bump();
        Ok(self.snapshot())
    }

    pub fn record_capture(
        &mut self,
        operation_id: &str,
        capture_ms: u64,
        speech_ms: u64,
        incomplete: bool,
    ) -> Result<(), VoiceError> {
        let operation = self.matching_operation_mut(operation_id)?;
        operation.capture_ms = capture_ms;
        operation.speech_ms = speech_ms.min(capture_ms);
        operation.capture_incomplete |= incomplete;
        Ok(())
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
                capture_incomplete: item.capture_incomplete,
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
            }),
            trial_result: self.trial_result.as_ref().map(|item| VoiceTrialResult {
                trial_id: item.trial_id.clone(),
                text: item.text.clone(),
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
