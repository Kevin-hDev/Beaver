use super::emergency::{EmergencyInventory, EmergencyKey};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum EmergencyHandoffReason {
    ReapFailed,
}

pub(crate) struct EmergencyRegistration {
    pub(crate) inventory: EmergencyInventory,
    pub(crate) key: Option<EmergencyKey>,
}

impl EmergencyRegistration {
    #[cfg(test)]
    pub(crate) fn key_for_test(&self) -> EmergencyKey {
        self.key.expect("emergency registration key")
    }

    pub(crate) fn hand_off_to_watchdog(mut self, _reason: EmergencyHandoffReason) {
        self.key.take();
    }
}

impl Drop for EmergencyRegistration {
    fn drop(&mut self) {
        if let Some(key) = self.key.take() {
            let _ = self.inventory.clear(key);
        }
    }
}
