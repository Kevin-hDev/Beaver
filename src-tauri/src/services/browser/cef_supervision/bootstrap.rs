use super::constants::{CEF_ADMISSION_TIMEOUT, CEF_HELPER_WAIT_SLICE};
use super::windows::{WindowsHelperObjects, WindowsProcessProbe};
use super::{CefIpcNames, CefLaunchMarker, CefUnavailableCategory};
use std::time::{Duration, Instant};

#[derive(Debug)]
pub(crate) struct WindowsHelperAdmission {
    objects: WindowsHelperObjects,
    generation: u64,
}

impl WindowsHelperAdmission {
    pub(crate) fn prepare(encoded_marker: &str) -> Result<Self, ()> {
        Self::prepare_with_timeout(encoded_marker, CEF_ADMISSION_TIMEOUT).map_err(|_| ())
    }

    fn prepare_with_timeout(
        encoded_marker: &str,
        timeout: Duration,
    ) -> Result<Self, CefUnavailableCategory> {
        let marker = CefLaunchMarker::decode_unique(&[encoded_marker])
            .map_err(|_| CefUnavailableCategory::Admission)?;
        let names =
            CefIpcNames::from_marker(&marker).map_err(|_| CefUnavailableCategory::Object)?;
        let objects = WindowsHelperObjects::open(&names)?;
        let pid = std::process::id();
        let started_at = WindowsProcessProbe::read(pid)?.started_at();
        objects
            .publish(marker.generation(), pid, started_at, 0)
            .map_err(|_| CefUnavailableCategory::Admission)?;
        wait_for_parent(&objects, marker.generation(), timeout)?;
        let admission = Self {
            objects,
            generation: marker.generation(),
        };
        admission.revalidate_category()?;
        Ok(admission)
    }

    pub(crate) fn revalidate(&self) -> Result<(), ()> {
        self.revalidate_category().map_err(|_| ())
    }

    fn revalidate_category(&self) -> Result<(), CefUnavailableCategory> {
        let control = self
            .objects
            .control_snapshot()
            .map_err(|_| CefUnavailableCategory::Admission)?;
        if control.closing || control.generation != self.generation {
            Err(CefUnavailableCategory::Admission)
        } else {
            Ok(())
        }
    }
}

fn wait_for_parent(
    objects: &WindowsHelperObjects,
    generation: u64,
    timeout: Duration,
) -> Result<(), CefUnavailableCategory> {
    let deadline = Instant::now()
        .checked_add(timeout)
        .ok_or(CefUnavailableCategory::Admission)?;
    loop {
        let control = objects
            .control_snapshot()
            .map_err(|_| CefUnavailableCategory::Admission)?;
        if control.closing || control.generation != generation {
            return Err(CefUnavailableCategory::Admission);
        }
        if objects.wait_for_closing(0)? {
            return Err(CefUnavailableCategory::Admission);
        }
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            return Err(CefUnavailableCategory::Admission);
        }
        let slice = remaining.min(CEF_HELPER_WAIT_SLICE);
        if objects.wait_for_admission(duration_millis(slice))? {
            return Ok(());
        }
    }
}

fn duration_millis(duration: Duration) -> u32 {
    duration.as_millis().clamp(1, u128::from(u32::MAX)) as u32
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::browser::cef_supervision::windows::WindowsPublicationObjects;
    use crate::services::browser::cef_supervision::CefProcessRole;

    #[test]
    fn helper_publishes_before_parent_admission() {
        let marker = CefLaunchMarker::generate(0, 41, CefProcessRole::Helper).expect("marker");
        let names = CefIpcNames::from_marker(&marker).expect("names");
        let parent = WindowsPublicationObjects::create(&names, 41).expect("objects");
        let encoded = marker.encode();
        let worker = std::thread::spawn(move || {
            WindowsHelperAdmission::prepare_with_timeout(&encoded, Duration::from_secs(1))
        });
        let deadline = Instant::now() + Duration::from_secs(1);
        while parent.mailbox_snapshot().is_err() && Instant::now() < deadline {
            std::thread::yield_now();
        }
        let published = parent.mailbox_snapshot().expect("published mailbox");
        assert_eq!(published.generation, 41);
        parent.signal_admission().expect("admission signal");
        worker
            .join()
            .expect("helper thread")
            .expect("admitted helper");
    }

    #[test]
    fn helper_refuses_a_closing_parent_before_admission() {
        let marker = CefLaunchMarker::generate(0, 42, CefProcessRole::Helper).expect("marker");
        let names = CefIpcNames::from_marker(&marker).expect("names");
        let parent = WindowsPublicationObjects::create(&names, 42).expect("objects");
        parent.begin_closing(1).expect("closing signal");
        let result = WindowsHelperAdmission::prepare_with_timeout(
            marker.encode().as_str(),
            Duration::from_millis(50),
        );
        assert_eq!(
            result.expect_err("must fail"),
            CefUnavailableCategory::Admission
        );
    }
}
