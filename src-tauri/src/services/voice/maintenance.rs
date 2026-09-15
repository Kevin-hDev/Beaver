use super::{actions::VoiceCoordinator, contracts::VoiceSnapshot};

pub fn sweep(coordinator: &mut VoiceCoordinator, now_ms: u64) -> VoiceSnapshot {
    coordinator.expire_at(now_ms);
    coordinator.snapshot()
}

use super::errors::VoiceError;

impl VoiceCoordinator {
    pub fn expire_at(&mut self, now_ms: u64) {
        let expired = self
            .recovery
            .as_ref()
            .is_some_and(|item| item.is_expired_at(now_ms));
        let delivery_expired = self
            .delivery
            .as_ref()
            .is_some_and(|item| item.is_expired_at(now_ms));
        if expired {
            self.recovery.take();
        }
        if delivery_expired {
            self.delivery.take();
            self.error = Some(VoiceError::configuration_unavailable());
        }
        if expired || delivery_expired {
            self.bump();
        }
    }
}
