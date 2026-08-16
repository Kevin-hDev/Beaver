use super::support::{close, pipe, stable_executable_link};
use std::os::unix::fs::MetadataExt;

#[cfg(target_os = "macos")]
#[test]
fn macos_gate_pipe_does_not_link_the_linux_only_pipe2_symbol() {
    let source = include_str!("spawn_gate_unix_support.rs");

    assert!(!source.contains("link_name = \"pipe2\""));
}

#[test]
fn gate_pipe_ends_are_close_on_exec_before_concurrent_spawns() {
    std::thread::scope(|scope| {
        let workers = (0..16)
            .map(|_| {
                scope.spawn(|| {
                    let (read_fd, write_fd) = pipe().expect("pipe");
                    let read_flags = unsafe { libc::fcntl(read_fd, libc::F_GETFD) };
                    let write_flags = unsafe { libc::fcntl(write_fd, libc::F_GETFD) };
                    let child = unsafe { libc::fork() };
                    assert!(child >= 0);
                    if child == 0 {
                        let path = b"/usr/bin/true\0";
                        let args = [path.as_ptr().cast(), std::ptr::null()];
                        unsafe {
                            libc::execve(path.as_ptr().cast(), args.as_ptr(), std::ptr::null())
                        };
                        unsafe { libc::_exit(127) };
                    }
                    let mut status = 0;
                    assert_eq!(unsafe { libc::waitpid(child, &mut status, 0) }, child);
                    assert!(libc::WIFEXITED(status));
                    close(read_fd);
                    close(write_fd);
                    (read_flags, write_flags)
                })
            })
            .collect::<Vec<_>>();
        for worker in workers {
            let (read_flags, write_flags) = worker.join().expect("worker");
            assert_ne!(read_flags & libc::FD_CLOEXEC, 0);
            assert_ne!(write_flags & libc::FD_CLOEXEC, 0);
        }
    });
}

#[test]
fn stale_gate_links_are_cleaned_before_next_creation() {
    let root = tempfile::tempdir().expect("root");
    let executable = root.path().join("ollama");
    std::fs::copy("/usr/bin/true", &executable).expect("executable");
    let stale = root.path().join(".beaver-gated-stale");
    std::fs::create_dir(&stale).expect("stale directory");
    std::fs::write(stale.join(".owner"), "4294967295").expect("owner");
    let metadata = std::fs::metadata(&executable).expect("metadata");
    let identity = (u128::from(metadata.dev()) << 64) | u128::from(metadata.ino());
    let link = stable_executable_link(&executable, identity).expect("link");
    assert_eq!(link.path().parent(), executable.parent());
    assert!(!stale.exists());
    drop(link);
    assert!(!root.path().join(".beaver-gated-stale").exists());
}

#[test]
fn stale_same_directory_gate_link_is_cleaned_before_next_creation() {
    let root = tempfile::tempdir().expect("root");
    let executable = root.path().join("ollama");
    std::fs::copy("/usr/bin/true", &executable).expect("executable");
    let stale = root.path().join(".beaver-gated-4294967295-stale");
    std::fs::hard_link(&executable, &stale).expect("stale hard link");
    let metadata = std::fs::metadata(&executable).expect("metadata");
    let identity = (u128::from(metadata.dev()) << 64) | u128::from(metadata.ino());
    let link = stable_executable_link(&executable, identity).expect("replacement link");
    assert!(!stale.exists());
    assert_eq!(link.path().parent(), executable.parent());
}

#[test]
fn stale_gate_cleanup_is_bounded_and_fails_closed() {
    let root = tempfile::tempdir().expect("root");
    let executable = root.path().join("ollama");
    std::fs::copy("/usr/bin/true", &executable).expect("executable");
    for index in 0..33 {
        let stale = root.path().join(format!(".beaver-gated-{index}"));
        std::fs::create_dir(&stale).expect("stale directory");
        std::fs::write(stale.join(".owner"), "4294967295").expect("owner");
    }
    let metadata = std::fs::metadata(&executable).expect("metadata");
    let identity = (u128::from(metadata.dev()) << 64) | u128::from(metadata.ino());
    assert!(stable_executable_link(&executable, identity).is_err());
}

#[test]
fn live_gate_owner_is_never_removed_by_recovery() {
    let root = tempfile::tempdir().expect("root");
    let executable = root.path().join("ollama");
    std::fs::copy("/usr/bin/true", &executable).expect("executable");
    let metadata = std::fs::metadata(&executable).expect("metadata");
    let identity = (u128::from(metadata.dev()) << 64) | u128::from(metadata.ino());
    let live = stable_executable_link(&executable, identity).expect("live link");
    let second = stable_executable_link(&executable, identity).expect("second link");
    assert!(live.path().exists());
    drop(second);
    drop(live);
}

#[test]
fn crashed_parent_gate_directory_is_recovered_on_next_creation() {
    let root = tempfile::tempdir().expect("root");
    let executable = root.path().join("ollama");
    std::fs::copy("/usr/bin/true", &executable).expect("executable");
    let stale = root.path().join(".beaver-gated-crashed");
    let child = unsafe { libc::fork() };
    assert!(child >= 0);
    if child == 0 {
        std::fs::create_dir(&stale).expect("stale directory");
        std::fs::write(stale.join(".owner"), std::process::id().to_string()).expect("owner");
        unsafe { libc::_exit(0) };
    }
    let mut status = 0;
    assert_eq!(unsafe { libc::waitpid(child, &mut status, 0) }, child);
    let metadata = std::fs::metadata(&executable).expect("metadata");
    let identity = (u128::from(metadata.dev()) << 64) | u128::from(metadata.ino());
    let link = stable_executable_link(&executable, identity).expect("recovered link");
    assert!(!stale.exists());
    drop(link);
}
