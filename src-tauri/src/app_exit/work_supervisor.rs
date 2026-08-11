#![cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "service producers adopt the app work supervisor during milestone 2"
    )
)]

use super::registry::AdmissionRegistry;
use super::{AppWorkAdmission, AppWorkAdmissionError};

#[derive(Clone)]
pub struct AppWorkSupervisor {
    registry: AdmissionRegistry,
}

impl AppWorkSupervisor {
    pub(super) fn new(registry: AdmissionRegistry) -> Self {
        Self { registry }
    }

    pub fn try_admit(&self) -> Result<AppWorkAdmission, AppWorkAdmissionError> {
        self.registry.try_admit()
    }
}
