use super::{
    actions::VoiceCoordinator,
    contracts::{VoiceDestination, VoiceSnapshot},
    errors::VoiceError,
    runtime::VoiceReservation,
    start_guards::validate_id,
    types::VoicePhase,
};

pub(super) struct Operation {
    pub(super) id: String,
    pub(super) destination: VoiceDestination,
    pub(super) context_generation: u64,
    pub(super) capture_ms: u64,
    pub(super) speech_ms: u64,
    pub(super) lost_samples: u64,
    pub(super) microphone_disconnected: bool,
    pub(super) level: f32,
    pub(super) cancelled: bool,
    pub(super) recovery_deleted: bool,
    pub(super) reservation: VoiceReservation,
}

impl VoiceCoordinator {
    pub fn note_microphone_disconnected(&mut self, operation_id: &str) -> Result<(), VoiceError> {
        validate_id(operation_id, super::limits::MAX_DESTINATION_ID_CHARS)?;
        let Some(operation) = self
            .operation
            .as_mut()
            .filter(|item| item.id == operation_id)
        else {
            return Err(VoiceError::invalid_transition());
        };
        operation.microphone_disconnected = true;
        self.bump();
        Ok(())
    }

    pub fn discard_operation(&mut self, operation_id: &str) -> Result<VoiceSnapshot, VoiceError> {
        validate_id(operation_id, super::limits::MAX_DESTINATION_ID_CHARS)?;
        if let Some(operation) = self.operation.as_mut() {
            if operation.id != operation_id {
                return Ok(self.snapshot());
            }
            if operation.reservation.context().phase() == VoicePhase::Stopping {
                return Ok(self.snapshot());
            }
            operation
                .reservation
                .context()
                .transition(VoicePhase::Stopping)?;
            operation.cancelled = true;
            operation.recovery_deleted = true;
            if self.recovery.as_ref().is_some_and(|recovery| {
                recovery.status == super::recovery::RecoveryStatus::Preparing
            }) {
                self.recovery.take();
            }
            ::log::info!("[voice] operation={operation_id} step=discarded");
            self.bump();
        } else if self.delivery.as_ref().is_some_and(|delivery| {
            delivery.id == operation_id || delivery.operation_id == operation_id
        }) {
            self.delivery.take();
            self.bump();
        }
        Ok(self.snapshot())
    }
}
