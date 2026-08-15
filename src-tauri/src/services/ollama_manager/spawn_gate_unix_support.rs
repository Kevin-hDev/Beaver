use super::super::process::OllamaProcessError;
use super::super::spawn_profile::OllamaSpawnAttempt;
use super::{NativeGatedProcess, OwnedProcessIdentity};
use std::ffi::CString;
use std::fs::{File, OpenOptions};
use std::os::fd::RawFd;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::MetadataExt;
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};

const GATE_LINK_PREFIX: &str = ".beaver-gated-";
const OWNER_FILE: &str = ".owner";
#[path = "spawn_gate_unix_support/cleanup.rs"]
mod cleanup;

pub(super) struct StableExecutableLink {
    directory: tempfile::TempDir,
    path: PathBuf,
}

impl StableExecutableLink {
    pub(super) fn path(&self) -> &Path {
        &self.path
    }
}

pub(super) fn stable_executable_link(
    executable: &Path,
    expected_identity: u128,
) -> Result<StableExecutableLink, OllamaProcessError> {
    let parent = executable.parent().ok_or(OllamaProcessError::Identity)?;
    let parent_file = open_parent(parent)?;
    cleanup::stale_gate_links(&parent_file, parent)?;
    let directory = tempfile::Builder::new()
        .prefix(GATE_LINK_PREFIX)
        .tempdir_in(parent)
        .map_err(|_| OllamaProcessError::Identity)?;
    std::fs::write(
        directory.path().join(OWNER_FILE),
        std::process::id().to_string(),
    )
    .map_err(|_| OllamaProcessError::Identity)?;
    let linked = directory.path().join("executable");
    std::fs::hard_link(executable, &linked).map_err(|_| OllamaProcessError::Identity)?;
    let metadata = std::fs::metadata(&linked).map_err(|_| OllamaProcessError::Identity)?;
    let actual = (u128::from(metadata.dev()) << 64) | u128::from(metadata.ino());
    (actual == expected_identity)
        .then_some(StableExecutableLink {
            directory,
            path: linked,
        })
        .ok_or(OllamaProcessError::Identity)
}

fn open_parent(parent: &Path) -> Result<File, OllamaProcessError> {
    OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC)
        .open(parent)
        .map_err(|_| OllamaProcessError::Identity)
}

pub(super) fn child_exec(
    read_fd: RawFd,
    write_fd: RawFd,
    cwd: *const libc::c_char,
    executable_path: *const libc::c_char,
    args: *const *const libc::c_char,
    environment: *const *const libc::c_char,
    stdio_fd: RawFd,
) -> ! {
    close(write_fd);
    unsafe {
        if libc::setpgid(0, 0) != 0 {
            libc::_exit(126);
        }
        if libc::dup2(stdio_fd, libc::STDOUT_FILENO) < 0
            || libc::dup2(stdio_fd, libc::STDERR_FILENO) < 0
        {
            libc::_exit(126);
        }
        #[cfg(target_os = "linux")]
        if libc::prctl(libc::PR_SET_PDEATHSIG, libc::SIGKILL) != 0 {
            libc::_exit(126);
        }
        if libc::getppid() == 1 || libc::chdir(cwd) != 0 {
            libc::_exit(126);
        }
        let mut byte = [0_u8; 1];
        if libc::read(read_fd, byte.as_mut_ptr().cast(), 1) != 1 {
            libc::_exit(126);
        }
        close(read_fd);
        libc::execve(executable_path, args, environment);
        libc::_exit(127);
    }
}

pub(super) fn environment_block(
    attempt: &OllamaSpawnAttempt<'_>,
) -> Result<Vec<CString>, OllamaProcessError> {
    let mut values = Vec::new();
    for (key, value) in attempt.profile().environment().entries() {
        let mut bytes = key.as_bytes().to_vec();
        bytes.push(b'=');
        bytes.extend(value.as_bytes());
        values.push(CString::new(bytes).map_err(|_| OllamaProcessError::InvalidState)?);
    }
    values.push(
        CString::new(format!("OLLAMA_HOST=127.0.0.1:{}", attempt.port()))
            .map_err(|_| OllamaProcessError::InvalidState)?,
    );
    Ok(values)
}

pub(super) fn c_string(path: impl AsRef<Path>) -> Result<CString, OllamaProcessError> {
    CString::new(path.as_ref().as_os_str().as_bytes()).map_err(|_| OllamaProcessError::InvalidState)
}

pub(super) fn pipe() -> std::io::Result<(RawFd, RawFd)> {
    let mut fds = [-1; 2];
    #[cfg(target_os = "linux")]
    #[allow(unused_unsafe)]
    let result = unsafe { pipe2_cloexec(fds.as_mut_ptr()) };
    #[cfg(not(target_os = "linux"))]
    let result = unsafe { libc::pipe(fds.as_mut_ptr()) };
    if result != 0 {
        return Err(std::io::Error::last_os_error());
    }
    // Linux can create both descriptors atomically with CLOEXEC. Other Unix
    // targets do not expose pipe2 consistently, so close both ends if either
    // descriptor cannot be made close-on-exec.
    #[cfg(not(target_os = "linux"))]
    for fd in fds {
        let flags = unsafe { libc::fcntl(fd, libc::F_GETFD) };
        if flags < 0 || unsafe { libc::fcntl(fd, libc::F_SETFD, flags | libc::FD_CLOEXEC) } < 0 {
            close(fds[0]);
            close(fds[1]);
            return Err(std::io::Error::last_os_error());
        }
    }
    Ok((fds[0], fds[1]))
}

#[cfg(target_os = "linux")]
unsafe fn pipe2_cloexec(fds: *mut RawFd) -> libc::c_int {
    libc::pipe2(fds, libc::O_CLOEXEC)
}

pub(super) fn close(fd: RawFd) {
    unsafe { libc::close(fd) };
}

pub(super) fn failed_process(pid: libc::pid_t, write_fd: RawFd) -> NativeGatedProcess {
    close(write_fd);
    NativeGatedProcess {
        pid,
        gate: None,
        #[cfg(test)]
        test_gate_keepalive: None,
        #[cfg(test)]
        force_reap_failure: false,
        exec_link: None,
        identity: OwnedProcessIdentity {
            pid: pid as u32,
            native_scope: pid as u64,
            native_start_time: 1,
            executable: 1,
        },
        opened: false,
        reaped: false,
    }
}

pub(super) fn wait_nonblocking(pid: libc::pid_t) -> Result<Option<i32>, OllamaProcessError> {
    let mut status = 0;
    let result = unsafe { libc::waitpid(pid, &mut status, libc::WNOHANG) };
    if result == pid {
        Ok(Some(status))
    } else if result == 0 {
        Ok(None)
    } else {
        Err(OllamaProcessError::Reap)
    }
}

pub(super) fn wait_blocking(pid: libc::pid_t) -> Result<(), OllamaProcessError> {
    let mut status = 0;
    (unsafe { libc::waitpid(pid, &mut status, 0) } == pid)
        .then_some(())
        .ok_or(OllamaProcessError::Reap)
}
