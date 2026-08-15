use super::{AppEmergencyPublisher, AppExitCoordinator};

impl AppExitCoordinator {
    #[allow(dead_code)]
    pub(crate) fn emergency_publisher(&self) -> AppEmergencyPublisher {
        AppEmergencyPublisher::new(self.emergency.clone())
    }
}
