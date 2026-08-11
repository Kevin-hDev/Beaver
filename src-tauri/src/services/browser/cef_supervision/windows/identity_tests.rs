use super::windows::{WindowsProcessIdentity, WindowsProcessProbe, CEF_PROCESS_ACCESS_RIGHTS};
use super::CefUnavailableCategory;
use std::process::{Child, Command, Stdio};

#[test]
fn process_access_is_the_exact_reviewed_minimum() {
    assert_eq!(CEF_PROCESS_ACCESS_RIGHTS, 0x0010_1101);
}

#[test]
fn a_native_identity_requires_parent_start_time_and_executable_to_match() {
    let child = ChildGuard::spawn();
    let probe = WindowsProcessProbe::read(child.id()).expect("probe");
    let identity = WindowsProcessIdentity::acquire(
        child.id(),
        std::process::id(),
        probe.started_at(),
        probe.executable(),
    )
    .expect("identity");

    assert_eq!(identity.pid(), child.id());
    assert_eq!(identity.started_at(), probe.started_at());
    assert_eq!(identity.parent_pid(), std::process::id());
    assert!(!identity.is_exited().expect("liveness"));

    assert_eq!(
        WindowsProcessIdentity::acquire(
            child.id(),
            std::process::id().wrapping_add(1),
            probe.started_at(),
            probe.executable(),
        )
        .unwrap_err(),
        CefUnavailableCategory::Admission
    );
    assert_eq!(
        WindowsProcessIdentity::acquire(
            child.id(),
            std::process::id(),
            probe.started_at().wrapping_add(1),
            probe.executable(),
        )
        .unwrap_err(),
        CefUnavailableCategory::Admission
    );
    let wrong_executable = std::env::current_exe().expect("test executable");
    assert_eq!(
        WindowsProcessIdentity::acquire(
            child.id(),
            std::process::id(),
            probe.started_at(),
            &wrong_executable,
        )
        .unwrap_err(),
        CefUnavailableCategory::Admission
    );
    assert!(WindowsProcessProbe::read(child.id()).is_ok());
}

pub(super) struct ChildGuard(Child);

impl ChildGuard {
    pub(super) fn spawn() -> Self {
        let child = Command::new("powershell.exe")
            .args([
                "-NoLogo",
                "-NoProfile",
                "-NonInteractive",
                "-Command",
                "Start-Sleep -Seconds 30",
            ])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("spawn fixed test child");
        Self(child)
    }

    pub(super) fn id(&self) -> u32 {
        self.0.id()
    }
}

impl Drop for ChildGuard {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}
