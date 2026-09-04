#[cfg(target_os = "macos")]
use super::cef_supervision::MacCefTrackerHandle as NativeCefTrackerHandle;
#[cfg(target_os = "windows")]
use super::cef_supervision::WindowsCefTrackerHandle as NativeCefTrackerHandle;
use super::cef_supervision::{CefUnavailableCategory, CEF_ADMISSION_SWITCH};
use cef::{CefString, CommandLine, ImplCommandLine};
use zeroize::Zeroizing;

#[derive(Clone, Debug)]
pub(super) struct BrowserCefSupervision {
    tracker: NativeCefTrackerHandle,
}

impl BrowserCefSupervision {
    pub(super) fn new(tracker: NativeCefTrackerHandle) -> Self {
        Self { tracker }
    }

    pub(super) fn attach_launch_marker(
        &self,
        command_line: Option<&mut CommandLine>,
    ) -> Result<(), CefUnavailableCategory> {
        let command_line = command_line.ok_or(CefUnavailableCategory::Admission)?;
        if command_line.is_valid() != 1 || command_line.is_read_only() != 0 {
            return self.fail(CefUnavailableCategory::Admission);
        }
        #[cfg(target_os = "windows")]
        self.remove_parent_safe_mode(command_line)?;
        let name = CefString::from(CEF_ADMISSION_SWITCH);
        if command_line.has_switch(Some(&name)) != 0 {
            return self.fail(CefUnavailableCategory::Admission);
        }
        let ticket = self
            .tracker
            .reserve()
            .inspect_err(|category| self.tracker.fail(*category))?;
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
    fn remove_parent_safe_mode(
        &self,
        command_line: &CommandLine,
    ) -> Result<(), CefUnavailableCategory> {
        let name = CefString::from(crate::services::extensions::cef_safe_mode_switch_name());
        let present = command_line.has_switch(Some(&name)) == 1;
        let value = command_line.switch_value(Some(&name));
        let value = Zeroizing::new(CefString::from(&value).to_string());
        let remove = match crate::services::extensions::cef_child_safe_mode_action(present, &value)
        {
            Ok(remove) => remove,
            Err(_) => return self.fail(CefUnavailableCategory::Admission),
        };
        if remove {
            command_line.remove_switch(Some(&name));
            if command_line.has_switch(Some(&name)) != 0 {
                return self.fail(CefUnavailableCategory::Admission);
            }
        }
        Ok(())
    }

    pub(super) fn fail<T>(
        &self,
        category: CefUnavailableCategory,
    ) -> Result<T, CefUnavailableCategory> {
        self.tracker.fail(category);
        Err(category)
    }
}
