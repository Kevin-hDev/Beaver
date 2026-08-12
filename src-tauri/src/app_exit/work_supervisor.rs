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
