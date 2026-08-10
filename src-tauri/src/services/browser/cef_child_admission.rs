#[cfg(target_os = "windows")]
use super::cef_supervision::WindowsCefTrackerHandle;
#[cfg(target_os = "windows")]
use super::cef_supervision::{CefUnavailableCategory, CEF_ADMISSION_SWITCH};
#[cfg(target_os = "windows")]
use cef::{CefString, CommandLine, ImplCommandLine};
#[cfg(target_os = "windows")]
use zeroize::Zeroizing;

#[derive(Clone, Debug)]
pub(super) struct BrowserCefSupervision {
    #[cfg(target_os = "windows")]
    tracker: WindowsCefTrackerHandle,
}

impl BrowserCefSupervision {
    pub(super) fn new(#[cfg(target_os = "windows")] tracker: WindowsCefTrackerHandle) -> Self {
        Self {
            #[cfg(target_os = "windows")]
            tracker,
        }
    }

    #[cfg(target_os = "windows")]
    pub(super) fn attach_launch_marker(
        &self,
        command_line: Option<&mut CommandLine>,
    ) -> Result<(), CefUnavailableCategory> {
        let command_line = command_line.ok_or(CefUnavailableCategory::Admission)?;
        if command_line.is_valid() != 1 || command_line.is_read_only() != 0 {
            return self.fail(CefUnavailableCategory::Admission);
        }
        let name = CefString::from(CEF_ADMISSION_SWITCH);
        if command_line.has_switch(Some(&name)) != 0 {
            return self.fail(CefUnavailableCategory::Admission);
        }
        let ticket = self.tracker.reserve().map_err(|category| {
            self.tracker.fail(category);
            category
        })?;
        let value = CefString::from(ticket.encoded_marker());
        command_line.append_switch_with_value(Some(&name), Some(&value));
        let copied = command_line.switch_value(Some(&name));
        let copied = Zeroizing::new(CefString::from(&copied).to_string());
        if command_line.has_switch(Some(&name)) != 1
            || !ticket.constant_time_encoded_matches(&copied)
        {
            return self.fail(CefUnavailableCategory::Admission);
        }
        Ok(())
    }

    #[cfg(target_os = "windows")]
    pub(super) fn fail<T>(
        &self,
        category: CefUnavailableCategory,
    ) -> Result<T, CefUnavailableCategory> {
        self.tracker.fail(category);
        Err(category)
    }
}
