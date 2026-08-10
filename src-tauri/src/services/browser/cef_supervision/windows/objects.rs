use super::super::shared_layout::{CefControlSnapshot, CefMailboxSnapshot};
use super::super::{
    CefControlPage, CefIpcNames, CefMailboxPage, CefSharedLayoutError, CefUnavailableCategory,
};
use super::handle::OwnedHandle;
use super::mapping::SharedMapping;
use super::security::{WindowsObjectKind, WindowsObjectSecurity};
use windows_sys::Win32::Foundation::{
    GetLastError, ERROR_ALREADY_EXISTS, WAIT_OBJECT_0, WAIT_TIMEOUT,
};
use windows_sys::Win32::System::Threading::{
    CreateEventW, OpenEventW, SetEvent, WaitForSingleObject,
};
use zeroize::Zeroizing;

const NATIVE_NAME_LIMIT: usize = 128;

#[derive(Debug)]
pub(in crate::services::browser) struct WindowsPublicationObjects {
    mailbox: SharedMapping<CefMailboxPage>,
    _control: SharedMapping<CefControlPage>,
    admission: OwnedHandle,
    _closing: OwnedHandle,
}

impl WindowsPublicationObjects {
    pub(in crate::services::browser) fn create(
        names: &CefIpcNames,
        generation: u64,
    ) -> Result<Self, CefUnavailableCategory> {
        let mailbox = SharedMapping::create(
            WindowsObjectKind::Mailbox,
            &wide_name(names, WindowsObjectKind::Mailbox)?,
            CefMailboxPage::new(),
        )?;
        let control = SharedMapping::create(
            WindowsObjectKind::Control,
            &wide_name(names, WindowsObjectKind::Control)?,
            CefControlPage::new(generation).map_err(|_| CefUnavailableCategory::Object)?,
        )?;
        let admission = create_event(names, WindowsObjectKind::AdmissionEvent)?;
        let closing = create_event(names, WindowsObjectKind::ClosingEvent)?;
        Ok(Self {
            mailbox,
            _control: control,
            admission,
            _closing: closing,
        })
    }

    pub(in crate::services::browser) fn mailbox_snapshot(
        &self,
    ) -> Result<CefMailboxSnapshot, CefSharedLayoutError> {
        self.mailbox.value().snapshot()
    }

    pub(in crate::services::browser) fn signal_admission(
        &self,
    ) -> Result<(), CefUnavailableCategory> {
        signal(&self.admission)
    }

    #[cfg(test)]
    pub(in crate::services::browser) fn begin_closing(
        &self,
        deadline_ticks: u64,
    ) -> Result<(), CefUnavailableCategory> {
        self._control
            .value()
            .begin_closing(deadline_ticks)
            .map_err(|_| CefUnavailableCategory::Object)?;
        signal(&self._closing)
    }

    #[cfg(test)]
    pub(in crate::services::browser) fn handles_are_non_inheritable(&self) -> bool {
        self.mailbox.handle_is_non_inheritable()
            && self._control.handle_is_non_inheritable()
            && self.admission.is_non_inheritable()
            && self._closing.is_non_inheritable()
    }
}

#[derive(Debug)]
pub(in crate::services::browser) struct WindowsHelperObjects {
    mailbox: SharedMapping<CefMailboxPage>,
    control: SharedMapping<CefControlPage>,
    admission: OwnedHandle,
    closing: OwnedHandle,
}

impl WindowsHelperObjects {
    pub(in crate::services::browser) fn open(
        names: &CefIpcNames,
    ) -> Result<Self, CefUnavailableCategory> {
        Ok(Self {
            mailbox: SharedMapping::open(
                WindowsObjectKind::Mailbox,
                &wide_name(names, WindowsObjectKind::Mailbox)?,
            )?,
            control: SharedMapping::open(
                WindowsObjectKind::Control,
                &wide_name(names, WindowsObjectKind::Control)?,
            )?,
            admission: open_event(names, WindowsObjectKind::AdmissionEvent)?,
            closing: open_event(names, WindowsObjectKind::ClosingEvent)?,
        })
    }

    pub(in crate::services::browser) fn publish(
        &self,
        generation: u64,
        pid: u32,
        started_at: u64,
        native_group: u32,
    ) -> Result<(), CefSharedLayoutError> {
        self.mailbox
            .value()
            .publish(generation, pid, started_at, native_group)
    }

    pub(in crate::services::browser) fn control_snapshot(
        &self,
    ) -> Result<CefControlSnapshot, CefSharedLayoutError> {
        self.control.value().snapshot()
    }

    pub(in crate::services::browser) fn wait_for_admission(
        &self,
        timeout_ms: u32,
    ) -> Result<bool, CefUnavailableCategory> {
        wait(&self.admission, timeout_ms)
    }

    pub(in crate::services::browser) fn wait_for_closing(
        &self,
        timeout_ms: u32,
    ) -> Result<bool, CefUnavailableCategory> {
        wait(&self.closing, timeout_ms)
    }

    #[cfg(test)]
    pub(in crate::services::browser) fn handles_are_non_inheritable(&self) -> bool {
        self.mailbox.handle_is_non_inheritable()
            && self.control.handle_is_non_inheritable()
            && self.admission.is_non_inheritable()
            && self.closing.is_non_inheritable()
    }
}

fn wide_name(
    names: &CefIpcNames,
    kind: WindowsObjectKind,
) -> Result<Zeroizing<Vec<u16>>, CefUnavailableCategory> {
    let name = names
        .get(kind.index())
        .ok_or(CefUnavailableCategory::Object)?;
    let value = Zeroizing::new(format!("Local\\{name}"));
    if value.contains('\0') || value.encode_utf16().count() >= NATIVE_NAME_LIMIT {
        return Err(CefUnavailableCategory::Object);
    }
    Ok(Zeroizing::new(
        value.encode_utf16().chain(std::iter::once(0)).collect(),
    ))
}

fn create_event(
    names: &CefIpcNames,
    kind: WindowsObjectKind,
) -> Result<OwnedHandle, CefUnavailableCategory> {
    let security = WindowsObjectSecurity::new(kind)?;
    let attributes = security.attributes();
    let name = wide_name(names, kind)?;
    let handle = OwnedHandle::new(unsafe { CreateEventW(&attributes, 1, 0, name.as_ptr()) })?;
    if unsafe { GetLastError() } == ERROR_ALREADY_EXISTS {
        Err(CefUnavailableCategory::Object)
    } else {
        Ok(handle)
    }
}

fn open_event(
    names: &CefIpcNames,
    kind: WindowsObjectKind,
) -> Result<OwnedHandle, CefUnavailableCategory> {
    let name = wide_name(names, kind)?;
    OwnedHandle::new(unsafe { OpenEventW(kind.helper_access(), 0, name.as_ptr()) })
}

fn signal(handle: &OwnedHandle) -> Result<(), CefUnavailableCategory> {
    if unsafe { SetEvent(handle.raw()) } == 0 {
        Err(CefUnavailableCategory::Object)
    } else {
        Ok(())
    }
}

fn wait(handle: &OwnedHandle, timeout_ms: u32) -> Result<bool, CefUnavailableCategory> {
    match unsafe { WaitForSingleObject(handle.raw(), timeout_ms) } {
        WAIT_OBJECT_0 => Ok(true),
        WAIT_TIMEOUT => Ok(false),
        _ => Err(CefUnavailableCategory::Object),
    }
}
