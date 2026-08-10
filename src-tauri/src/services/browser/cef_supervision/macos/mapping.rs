use super::super::CefUnavailableCategory;
use std::fs::{File, OpenOptions};
use std::marker::PhantomData;
use std::os::fd::AsRawFd;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::{MetadataExt, OpenOptionsExt};
use std::path::Path;
use zeroize::Zeroizing;

pub(super) struct MacMapping<T> {
    file: MacFile,
    address: *mut T,
    _value: PhantomData<T>,
}

impl<T> MacMapping<T> {
    pub(super) fn create(
        path: Zeroizing<Vec<u8>>,
        value: T,
    ) -> Result<Self, CefUnavailableCategory> {
        let file = MacFile::create(path)?;
        file.file
            .set_len(std::mem::size_of::<T>() as u64)
            .map_err(|_| CefUnavailableCategory::Object)?;
        let mapping = Self::map(file, true)?;
        unsafe { mapping.address.write(value) };
        Ok(mapping)
    }

    pub(super) fn open(
        path: &Zeroizing<Vec<u8>>,
        writable: bool,
    ) -> Result<Self, CefUnavailableCategory> {
        Self::map(MacFile::open(path, writable)?, writable)
    }

    fn map(file: MacFile, writable: bool) -> Result<Self, CefUnavailableCategory> {
        let metadata = file
            .file
            .metadata()
            .map_err(|_| CefUnavailableCategory::Object)?;
        if !metadata.is_file()
            || metadata.nlink() != 1
            || metadata.len() != std::mem::size_of::<T>() as u64
        {
            return Err(CefUnavailableCategory::Object);
        }
        let protection = libc::PROT_READ | if writable { libc::PROT_WRITE } else { 0 };
        let address = unsafe {
            libc::mmap(
                std::ptr::null_mut(),
                std::mem::size_of::<T>(),
                protection,
                libc::MAP_SHARED,
                file.file.as_raw_fd(),
                0,
            )
        };
        if address == libc::MAP_FAILED {
            return Err(CefUnavailableCategory::Object);
        }
        Ok(Self {
            file,
            address: address.cast(),
            _value: PhantomData,
        })
    }

    pub(super) fn value(&self) -> &T {
        unsafe { &*self.address }
    }

    pub(super) fn is_close_on_exec(&self) -> bool {
        let flags = unsafe { libc::fcntl(self.file.file.as_raw_fd(), libc::F_GETFD) };
        flags >= 0 && flags & libc::FD_CLOEXEC != 0
    }
}

impl<T> Drop for MacMapping<T> {
    fn drop(&mut self) {
        if !self.address.is_null() {
            unsafe { libc::munmap(self.address.cast(), std::mem::size_of::<T>()) };
        }
    }
}

impl<T> std::fmt::Debug for MacMapping<T> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("MacMapping([redacted])")
    }
}

struct MacFile {
    file: File,
    cleanup: Option<Zeroizing<Vec<u8>>>,
}

impl MacFile {
    fn create(path: Zeroizing<Vec<u8>>) -> Result<Self, CefUnavailableCategory> {
        let file = open_options(true)
            .create_new(true)
            .mode(0o600)
            .open(secret_path(&path))
            .map_err(|_| CefUnavailableCategory::Object)?;
        Ok(Self {
            file,
            cleanup: Some(path),
        })
    }

    fn open(path: &Zeroizing<Vec<u8>>, writable: bool) -> Result<Self, CefUnavailableCategory> {
        let file = open_options(writable)
            .open(secret_path(path))
            .map_err(|_| CefUnavailableCategory::Object)?;
        Ok(Self {
            file,
            cleanup: None,
        })
    }
}

impl Drop for MacFile {
    fn drop(&mut self) {
        if let Some(path) = self.cleanup.take() {
            let _ = std::fs::remove_file(secret_path(&path));
        }
    }
}

fn open_options(writable: bool) -> OpenOptions {
    let mut options = OpenOptions::new();
    options.read(true).write(writable);
    options.custom_flags(libc::O_CLOEXEC | libc::O_NOFOLLOW);
    options
}

fn secret_path(bytes: &[u8]) -> &Path {
    Path::new(std::ffi::OsStr::from_bytes(bytes))
}
