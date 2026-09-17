use std::ffi::CString;

pub(super) enum Entered {
    Child,
    Parent(i32),
    Unavailable,
}

pub(super) fn enter() -> Result<Entered, String> {
    let uid = unsafe { libc::getuid() };
    let gid = unsafe { libc::getgid() };
    let flags = libc::CLONE_NEWUSER | libc::CLONE_NEWNS | libc::CLONE_NEWPID;
    if unsafe { libc::unshare(flags) } != 0 {
        return Ok(Entered::Unavailable);
    }
    let _ = std::fs::write("/proc/self/setgroups", "deny");
    std::fs::write("/proc/self/uid_map", format!("0 {uid} 1\n")).map_err(|_| error())?;
    std::fs::write("/proc/self/gid_map", format!("0 {gid} 1\n")).map_err(|_| error())?;
    let pid = unsafe { libc::fork() };
    if pid < 0 {
        return Err(error());
    }
    if pid > 0 {
        return wait_for(pid).map(Entered::Parent);
    }
    mount_private_proc()?;
    drop_capabilities()?;
    Ok(Entered::Child)
}

fn mount_private_proc() -> Result<(), String> {
    let root = CString::new("/").map_err(|_| error())?;
    let proc_path = CString::new("/proc").map_err(|_| error())?;
    let proc_type = CString::new("proc").map_err(|_| error())?;
    let private = unsafe {
        libc::mount(
            std::ptr::null(),
            root.as_ptr(),
            std::ptr::null(),
            libc::MS_REC | libc::MS_PRIVATE,
            std::ptr::null(),
        )
    };
    if private != 0 {
        return Err(error());
    }
    let mounted = unsafe {
        libc::mount(
            proc_type.as_ptr(),
            proc_path.as_ptr(),
            proc_type.as_ptr(),
            libc::MS_NOSUID | libc::MS_NODEV | libc::MS_NOEXEC,
            std::ptr::null(),
        )
    };
    (mounted == 0).then_some(()).ok_or_else(error)
}

fn wait_for(pid: libc::pid_t) -> Result<i32, String> {
    let mut status = 0;
    loop {
        let result = unsafe { libc::waitpid(pid, &mut status, 0) };
        if result == pid {
            if libc::WIFEXITED(status) {
                return Ok(libc::WEXITSTATUS(status));
            }
            if libc::WIFSIGNALED(status) {
                return Ok(128 + libc::WTERMSIG(status));
            }
            return Err(error());
        }
        if result < 0 && std::io::Error::last_os_error().kind() == std::io::ErrorKind::Interrupted {
            continue;
        }
        return Err(error());
    }
}

fn drop_capabilities() -> Result<(), String> {
    #[repr(C)]
    struct Header {
        version: u32,
        pid: i32,
    }
    #[repr(C)]
    #[derive(Clone, Copy)]
    struct Data {
        effective: u32,
        permitted: u32,
        inheritable: u32,
    }
    const LINUX_CAPABILITY_VERSION_3: u32 = 0x2008_0522;
    let mut header = Header {
        version: LINUX_CAPABILITY_VERSION_3,
        pid: 0,
    };
    let mut data = [
        Data {
            effective: 0,
            permitted: 0,
            inheritable: 0,
        };
        2
    ];
    // Le namespace donne temporairement CAP_SYS_ADMIN pour monter /proc.
    // Les capacités sont retirées avant d'appliquer Landlock et d'exécuter l'outil.
    let result = unsafe {
        libc::syscall(
            libc::SYS_capset,
            std::ptr::from_mut(&mut header),
            data.as_mut_ptr(),
        )
    };
    (result == 0).then_some(()).ok_or_else(error)
}

fn error() -> String {
    super::launch::sandbox_error()
}
