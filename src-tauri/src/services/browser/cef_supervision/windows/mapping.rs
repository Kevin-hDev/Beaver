use super::super::CefUnavailableCategory;
use super::handle::OwnedHandle;
use super::security::{WindowsObjectKind, WindowsObjectSecurity};
use std::marker::PhantomData;
use windows_sys::Win32::Foundation::{GetLastError, ERROR_ALREADY_EXISTS, INVALID_HANDLE_VALUE};
use windows_sys::Win32::System::Memory::{
    CreateFileMappingW, MapViewOfFile, OpenFileMappingW, UnmapViewOfFile, FILE_MAP_ALL_ACCESS,
    PAGE_READWRITE,
};

pub(super) struct SharedMapping<T> {
    _handle: OwnedHandle,
    address: *mut T,
    _value: PhantomData<T>,
}

impl<T> SharedMapping<T> {
    pub(super) fn create(
        kind: WindowsObjectKind,
        name: &[u16],
        value: T,
    ) -> Result<Self, CefUnavailableCategory> {
        let security = WindowsObjectSecurity::new(kind)?;
        let attributes = security.attributes();
        let size =
            u32::try_from(std::mem::size_of::<T>()).map_err(|_| CefUnavailableCategory::Object)?;
        let raw = unsafe {
            CreateFileMappingW(
                INVALID_HANDLE_VALUE,
                &attributes,
                PAGE_READWRITE,
                0,
                size,
                name.as_ptr(),
            )
        };
        let handle = OwnedHandle::new(raw)?;
        if unsafe { GetLastError() } == ERROR_ALREADY_EXISTS {
            return Err(CefUnavailableCategory::Object);
        }
        let mapping = Self::map(handle, FILE_MAP_ALL_ACCESS)?;
        unsafe { mapping.address.write(value) };
        Ok(mapping)
    }

    pub(super) fn open(
        kind: WindowsObjectKind,
        name: &[u16],
    ) -> Result<Self, CefUnavailableCategory> {
        let access = kind.helper_access();
        let raw = unsafe { OpenFileMappingW(access, 0, name.as_ptr()) };
        Self::map(OwnedHandle::new(raw)?, access)
    }

    fn map(handle: OwnedHandle, access: u32) -> Result<Self, CefUnavailableCategory> {
        let view = unsafe { MapViewOfFile(handle.raw(), access, 0, 0, std::mem::size_of::<T>()) };
        if view.Value.is_null() {
            return Err(CefUnavailableCategory::Object);
        }
        Ok(Self {
            _handle: handle,
            address: view.Value.cast(),
            _value: PhantomData,
        })
    }

    pub(super) fn value(&self) -> &T {
        unsafe { &*self.address }
    }

    #[cfg(test)]
    pub(super) fn handle_is_non_inheritable(&self) -> bool {
        self._handle.is_non_inheritable()
    }
}

impl<T> Drop for SharedMapping<T> {
    fn drop(&mut self) {
        if !self.address.is_null() {
            unsafe {
                UnmapViewOfFile(
                    windows_sys::Win32::System::Memory::MEMORY_MAPPED_VIEW_ADDRESS {
                        Value: self.address.cast(),
                    },
                )
            };
        }
    }
}

impl<T> std::fmt::Debug for SharedMapping<T> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("SharedMapping([redacted])")
    }
}

// The mapped pages contain only atomics and the Windows mapping handle is thread-safe.
unsafe impl<T: Send> Send for SharedMapping<T> {}
unsafe impl<T: Sync> Sync for SharedMapping<T> {}
