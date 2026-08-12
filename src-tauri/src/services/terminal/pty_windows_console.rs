use std::fs::File;
use std::os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle};
use std::sync::Mutex;
use windows_sys::Win32::Foundation::HANDLE;
use windows_sys::Win32::System::Console::{
    ClosePseudoConsole, CreatePseudoConsole, ResizePseudoConsole, COORD, HPCON,
};
use windows_sys::Win32::System::Pipes::CreatePipe;

pub(super) struct PseudoConsole {
    handle: Mutex<Option<HPCON>>,
}

impl PseudoConsole {
    pub(super) fn create(cols: u16, rows: u16) -> Result<(Self, File, File), String> {
        let (input_read, input_write) = create_pipe()?;
        let (output_read, output_write) = create_pipe()?;
        let mut handle = 0;
        // SAFETY: all four handles are live owned pipe endpoints, the size was
        // validated by PtySession, and `handle` is a valid out-parameter.
        let result = unsafe {
            CreatePseudoConsole(
                coord(cols, rows),
                input_read.as_raw_handle(),
                output_write.as_raw_handle(),
                0,
                &mut handle,
            )
        };
        if result < 0 || handle == 0 {
            return Err(terminal_error());
        }
        drop(input_read);
        drop(output_write);
        Ok((
            Self {
                handle: Mutex::new(Some(handle)),
            },
            File::from(input_write),
            File::from(output_read),
        ))
    }

    pub(super) fn resize(&self, cols: u16, rows: u16) -> Result<(), String> {
        let handle = self.handle.lock().map_err(|_| terminal_error())?;
        let handle = handle.as_ref().ok_or_else(terminal_error)?;
        // SAFETY: the mutex keeps the owned pseudoconsole live for this call.
        if unsafe { ResizePseudoConsole(*handle, coord(cols, rows)) } < 0 {
            Err(terminal_error())
        } else {
            Ok(())
        }
    }

    pub(super) fn close(&self) {
        let handle = self.handle.lock().ok().and_then(|mut handle| handle.take());
        if let Some(handle) = handle {
            // Beaver closes conout and conin before this call. Microsoft names
            // those channels as the condition that keeps legacy close bounded.
            // SAFETY: taking the Option claims the live HPCON exactly once.
            unsafe { ClosePseudoConsole(handle) };
        }
    }
}

impl Drop for PseudoConsole {
    fn drop(&mut self) {
        self.close();
    }
}

// SAFETY: the mutex-owned nonzero HPCON remains unchanged for every borrow;
// spawning happens before the console is shared or eligible for close.
unsafe impl windows_spawn::AsPseudoConsole for PseudoConsole {
    fn raw_pseudoconsole(&self) -> isize {
        *self
            .handle
            .lock()
            .expect("pseudoconsole handle lock")
            .as_ref()
            .expect("live pseudoconsole handle")
    }
}

fn create_pipe() -> Result<(OwnedHandle, OwnedHandle), String> {
    let mut read: HANDLE = std::ptr::null_mut();
    let mut write: HANDLE = std::ptr::null_mut();
    // SAFETY: both out-pointers are valid and null security attributes request
    // non-inheritable anonymous pipe handles owned by this process.
    if unsafe { CreatePipe(&mut read, &mut write, std::ptr::null(), 0) } == 0 {
        return Err(terminal_error());
    }
    // SAFETY: CreatePipe succeeded and transferred exactly one owned reference
    // for each non-null raw handle returned above.
    Ok(unsafe {
        (
            OwnedHandle::from_raw_handle(read),
            OwnedHandle::from_raw_handle(write),
        )
    })
}

fn coord(cols: u16, rows: u16) -> COORD {
    COORD {
        X: cols as i16,
        Y: rows as i16,
    }
}

fn terminal_error() -> String {
    "terminal-error".to_string()
}
