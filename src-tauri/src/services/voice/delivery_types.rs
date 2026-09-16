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
    pub microphone_disconnected: bool,
}

pub struct TrialResult {
    pub trial_id: String,
    pub text: String,
    pub capture_ms: u64,
    pub compute_ms: u64,
    pub created_at_ms: u64,
}

impl TrialResult {
    pub fn is_expired_at(&self, now_ms: u64) -> bool {
        now_ms.saturating_sub(self.created_at_ms) >= limits::DELIVERY_TIMEOUT_MS
    }
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
            microphone_disconnected: false,
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
