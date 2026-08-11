use super::super::constants::publication_deadline;
use super::super::{CefIpcNames, CefLaunchTicket, CefProcessRole, CefUnavailableCategory};
use super::objects::WindowsPublicationObjects;
use super::tracker::WindowsCefTrackerHandle;
use super::tracker_pending::WindowsPendingLaunch;
use std::sync::atomic::Ordering;
use std::time::Instant;

impl WindowsCefTrackerHandle {
    pub(in crate::services::browser) fn reserve(
        &self,
    ) -> Result<CefLaunchTicket, CefUnavailableCategory> {
        self.reserve_until(publication_deadline(Instant::now()))
    }

    fn reserve_until(
        &self,
        expires_at: Instant,
    ) -> Result<CefLaunchTicket, CefUnavailableCategory> {
        let _permit = self
            .shared
            .gate
            .try_enter()
            .map_err(|_| CefUnavailableCategory::Admission)?;
        if self.failure().is_some() || self.shared.stopping.load(Ordering::Acquire) {
            return Err(CefUnavailableCategory::Admission);
        }
        let reservation = self
            .shared
            .table
            .try_reserve(CefProcessRole::Helper)
            .map_err(|_| CefUnavailableCategory::Admission)?;
        let names = CefIpcNames::from_marker(reservation.marker())
            .map_err(|_| CefUnavailableCategory::Object)?;
        let objects = WindowsPublicationObjects::create(&names, reservation.marker().generation())?;
        let ticket = CefLaunchTicket::new(reservation.marker());
        let slot = reservation.marker().slot();
        if self.shared.gate.is_closed() {
            return Err(CefUnavailableCategory::Admission);
        }
        self.shared.pending.install(
            slot,
            WindowsPendingLaunch {
                reservation,
                objects,
                expires_at,
            },
        )?;
        Ok(ticket)
    }

    #[cfg(test)]
    pub(in crate::services::browser) fn reserve_until_for_test(
        &self,
        expires_at: Instant,
    ) -> Result<CefLaunchTicket, CefUnavailableCategory> {
        self.reserve_until(expires_at)
    }

    pub(in crate::services::browser) fn failure(&self) -> Option<CefUnavailableCategory> {
        self.shared.failure()
    }

    pub(in crate::services::browser) fn fail(&self, category: CefUnavailableCategory) {
        self.shared.fail(category);
    }
}
