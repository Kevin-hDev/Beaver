use super::super::shared_layout::{CefControlSnapshot, CefMailboxSnapshot};
use super::super::{
    CefControlPage, CefEventPage, CefIpcNames, CefMailboxPage, CefSharedLayoutError,
    CefUnavailableCategory,
};
use super::mapping::MacMapping;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use zeroize::Zeroizing;

#[derive(Debug)]
pub(in crate::services::browser) struct MacPublicationObjects {
    mailbox: MacMapping<CefMailboxPage>,
    _control: MacMapping<CefControlPage>,
    admission: MacMapping<CefEventPage>,
    _closing: MacMapping<CefEventPage>,
}

impl MacPublicationObjects {
    pub(in crate::services::browser) fn create(
        root: &Path,
        names: &CefIpcNames,
        generation: u64,
    ) -> Result<Self, CefUnavailableCategory> {
        ensure_private_root(root)?;
        Ok(Self {
            mailbox: MacMapping::create(object_path(root, names, 0)?, CefMailboxPage::new())?,
            _control: MacMapping::create(
                object_path(root, names, 1)?,
                CefControlPage::new(generation).map_err(|_| CefUnavailableCategory::Object)?,
            )?,
            admission: MacMapping::create(object_path(root, names, 2)?, CefEventPage::new())?,
            _closing: MacMapping::create(object_path(root, names, 3)?, CefEventPage::new())?,
        })
    }

    pub(in crate::services::browser) fn mailbox_snapshot(
        &self,
    ) -> Result<CefMailboxSnapshot, CefSharedLayoutError> {
        self.mailbox.value().snapshot()
    }

    pub(in crate::services::browser) fn signal_admission(&self) {
        self.admission.value().signal();
    }

    pub(in crate::services::browser) fn begin_closing(
        &self,
        deadline_ticks: u64,
    ) -> Result<(), CefSharedLayoutError> {
        self._control.value().begin_closing(deadline_ticks)?;
        self._closing.value().signal();
        Ok(())
    }

    #[cfg(test)]
    pub(in crate::services::browser) fn descriptors_are_close_on_exec(&self) -> bool {
        self.mailbox.is_close_on_exec()
            && self._control.is_close_on_exec()
            && self.admission.is_close_on_exec()
            && self._closing.is_close_on_exec()
    }
}

#[derive(Debug)]
pub(in crate::services::browser) struct MacHelperObjects {
    mailbox: MacMapping<CefMailboxPage>,
    control: MacMapping<CefControlPage>,
    admission: MacMapping<CefEventPage>,
    closing: MacMapping<CefEventPage>,
}

impl MacHelperObjects {
    pub(in crate::services::browser) fn open(
        root: &Path,
        names: &CefIpcNames,
    ) -> Result<Self, CefUnavailableCategory> {
        Ok(Self {
            mailbox: MacMapping::open(&object_path(root, names, 0)?, true)?,
            control: MacMapping::open(&object_path(root, names, 1)?, false)?,
            admission: MacMapping::open(&object_path(root, names, 2)?, false)?,
            closing: MacMapping::open(&object_path(root, names, 3)?, false)?,
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

    pub(in crate::services::browser) fn admission_signaled(
        &self,
    ) -> Result<bool, CefSharedLayoutError> {
        self.admission.value().is_signaled()
    }

    pub(in crate::services::browser) fn closing_signaled(
        &self,
    ) -> Result<bool, CefSharedLayoutError> {
        self.closing.value().is_signaled()
    }

    #[cfg(test)]
    pub(in crate::services::browser) fn descriptors_are_close_on_exec(&self) -> bool {
        self.mailbox.is_close_on_exec()
            && self.control.is_close_on_exec()
            && self.admission.is_close_on_exec()
            && self.closing.is_close_on_exec()
    }
}

fn ensure_private_root(root: &Path) -> Result<(), CefUnavailableCategory> {
    match std::fs::symlink_metadata(root) {
        Ok(metadata) if metadata.is_dir() && !metadata.file_type().is_symlink() => {}
        Ok(_) => return Err(CefUnavailableCategory::Permission),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            std::fs::create_dir(root).map_err(|_| CefUnavailableCategory::Permission)?;
        }
        Err(_) => return Err(CefUnavailableCategory::Permission),
    }
    std::fs::set_permissions(root, std::fs::Permissions::from_mode(0o700))
        .map_err(|_| CefUnavailableCategory::Permission)
}

fn object_path(
    root: &Path,
    names: &CefIpcNames,
    index: usize,
) -> Result<Zeroizing<Vec<u8>>, CefUnavailableCategory> {
    let name = names.get(index).ok_or(CefUnavailableCategory::Object)?;
    if name.len() > 128
        || !name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
    {
        return Err(CefUnavailableCategory::Object);
    }
    let root = root.as_os_str().as_bytes();
    if root.len().saturating_add(name.len()).saturating_add(1) > 4096 {
        return Err(CefUnavailableCategory::Object);
    }
    let mut path = Zeroizing::new(Vec::with_capacity(root.len() + name.len() + 1));
    path.extend_from_slice(root);
    path.push(b'/');
    path.extend_from_slice(name.as_bytes());
    Ok(path)
}
