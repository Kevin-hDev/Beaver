use super::work_supervision::{ExtensionWorkAdmissionError, ExtensionWorkServices};
use crate::services::work_registry::ServiceWorkCancellation;
use std::future::Future;

impl ExtensionWorkServices {
    pub(super) fn spawn_event_worker<Factory, Task>(
        &self,
        work: Factory,
    ) -> Result<(), ExtensionWorkAdmissionError>
    where
        Factory: FnOnce(ServiceWorkCancellation) -> Task + Send + 'static,
        Task: Future + Send + 'static,
    {
        self.events
            .spawn(work)
            .map_err(super::work_supervision::map_admission_error)
    }

    pub(super) fn event_router(&self) -> &super::event_delivery::EventRouter {
        &self.event_router
    }
}
