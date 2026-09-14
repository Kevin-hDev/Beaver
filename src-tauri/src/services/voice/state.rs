use super::errors::VoiceError;
use super::types::VoicePhase;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CancelDisposition {
    Discard,
    Recover,
}

pub fn cancel_disposition(speech_ms: u64) -> CancelDisposition {
    if speech_ms >= super::limits::RECOVERY_SPEECH_THRESHOLD_MS {
        CancelDisposition::Recover
    } else {
        CancelDisposition::Discard
    }
}

#[derive(Debug)]
pub struct VoiceState {
    phase: VoicePhase,
}

impl Default for VoiceState {
    fn default() -> Self {
        Self {
            phase: VoicePhase::Idle,
        }
    }
}

impl VoiceState {
    pub fn phase(&self) -> VoicePhase {
        self.phase
    }

    pub fn begin(&mut self) -> Result<(), VoiceError> {
        if self.blocks_new_operation() {
            return Err(VoiceError::busy());
        }
        self.phase = VoicePhase::Preparing;
        Ok(())
    }

    pub fn transition(&mut self, next: VoicePhase) -> Result<(), VoiceError> {
        let allowed = matches!(
            (self.phase, next),
            (
                VoicePhase::Preparing,
                VoicePhase::Listening
                    | VoicePhase::Transcribing
                    | VoicePhase::Recovering
                    | VoicePhase::Stopping
            ) | (
                VoicePhase::Listening,
                VoicePhase::Transcribing | VoicePhase::Recovering | VoicePhase::Stopping
            ) | (
                VoicePhase::Transcribing,
                VoicePhase::Delivering | VoicePhase::Recovering | VoicePhase::Stopping
            ) | (VoicePhase::Recovering, VoicePhase::Stopping)
                | (VoicePhase::Delivering, VoicePhase::Recovering)
        );
        if !allowed {
            return Err(VoiceError::invalid_transition());
        }
        self.phase = next;
        Ok(())
    }

    pub fn finish(&mut self) {
        self.phase = VoicePhase::Idle;
    }

    pub fn blocks_new_operation(&self) -> bool {
        self.phase != VoicePhase::Idle
    }

    pub fn locks_models(&self) -> bool {
        matches!(
            self.phase,
            VoicePhase::Preparing
                | VoicePhase::Listening
                | VoicePhase::Transcribing
                | VoicePhase::Recovering
                | VoicePhase::Stopping
        )
    }

    #[cfg(test)]
    pub(super) fn set_phase_for_test(&mut self, phase: VoicePhase) {
        self.phase = phase;
    }
}
