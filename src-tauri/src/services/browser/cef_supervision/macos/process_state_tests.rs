use super::super::CefUnavailableCategory;
use super::process_state::{
    classify_signal_result, observe_with, revalidate_with, MacBsdObservation,
    MacExistenceObservation, MacKernelIdentity, MacProcessObservation, MacProcessProbe,
    MacSignalObservation, MacSignalResult, MacWaitObservation,
};
use super::MacProcessIdentity;
use std::path::PathBuf;

const PID: u32 = 42;
const EXECUTABLE: &str = "/Applications/Beaver.app/Contents/MacOS/cl-go-dash-helper";

#[test]
fn exact_process_classification_table() {
    let identity = identity();
    let equal = MacKernelIdentity::from_identity(&identity);
    let different = MacKernelIdentity {
        parent_pid: identity.parent_pid,
        started_at: identity.started_at + 1,
        process_group: identity.process_group,
    };
    let cases = [
        Case::new(
            "active identity without executable read",
            Probe::active(equal, Err(())),
            MacProcessObservation::Alive,
            MacSignalObservation::Unknown,
        ),
        Case::new(
            "complete active identity",
            Probe::active(equal, Ok(PathBuf::from(EXECUTABLE))),
            MacProcessObservation::Alive,
            MacSignalObservation::Ready,
        ),
        Case::new(
            "zombie",
            Probe::zombie(),
            MacProcessObservation::Stopped,
            MacSignalObservation::Stopped,
        ),
        Case::new(
            "reapable child",
            Probe::fallback(
                MacWaitObservation::Reapable,
                MacExistenceObservation::Present,
            ),
            MacProcessObservation::Stopped,
            MacSignalObservation::Stopped,
        ),
        Case::new(
            "missing pid",
            Probe::fallback(
                MacWaitObservation::NotReapable,
                MacExistenceObservation::Missing,
            ),
            MacProcessObservation::Stopped,
            MacSignalObservation::Stopped,
        ),
        Case::new(
            "different kernel identity",
            Probe::active(different, Ok(PathBuf::from(EXECUTABLE))),
            MacProcessObservation::Stopped,
            MacSignalObservation::Stopped,
        ),
        Case::new(
            "different active executable",
            Probe::active(equal, Ok(PathBuf::from("/tmp/other"))),
            MacProcessObservation::Alive,
            MacSignalObservation::Stopped,
        ),
        Case::new(
            "unreadable kernel fields",
            Probe::unreadable_kernel(),
            MacProcessObservation::Unknown,
            MacSignalObservation::Unknown,
        ),
        Case::new(
            "unreadable active executable",
            Probe::active(equal, Err(())),
            MacProcessObservation::Alive,
            MacSignalObservation::Unknown,
        ),
        Case::new(
            "present but unavailable pid",
            Probe::fallback(
                MacWaitObservation::NotReapable,
                MacExistenceObservation::Present,
            ),
            MacProcessObservation::Unknown,
            MacSignalObservation::Unknown,
        ),
        Case::new(
            "existence check denied",
            Probe::fallback(
                MacWaitObservation::NotReapable,
                MacExistenceObservation::Unknown,
            ),
            MacProcessObservation::Unknown,
            MacSignalObservation::Unknown,
        ),
    ];
    for case in cases {
        assert_eq!(
            observe_with(&case.probe, &identity),
            case.liveness,
            "{} liveness",
            case.name
        );
        assert_eq!(
            revalidate_with(&case.probe, &identity),
            case.before_signal,
            "{} before signal",
            case.name
        );
    }
}

#[test]
fn esrch_after_successful_revalidation_is_stopped() {
    assert_eq!(
        classify_signal_result(-1, Some(libc::ESRCH)),
        Ok(MacSignalResult::Stopped)
    );
}

#[test]
fn successful_signal_is_sent() {
    assert_eq!(classify_signal_result(0, None), Ok(MacSignalResult::Sent));
}

#[test]
fn non_esrch_signal_error_remains_fatal() {
    assert_eq!(
        classify_signal_result(-1, Some(libc::EPERM)),
        Err(CefUnavailableCategory::Reaper)
    );
}

struct Case {
    name: &'static str,
    probe: Probe,
    liveness: MacProcessObservation,
    before_signal: MacSignalObservation,
}

impl Case {
    fn new(
        name: &'static str,
        probe: Probe,
        liveness: MacProcessObservation,
        before_signal: MacSignalObservation,
    ) -> Self {
        Self {
            name,
            probe,
            liveness,
            before_signal,
        }
    }
}

struct Probe {
    bsd: MacBsdObservation,
    wait: MacWaitObservation,
    existence: MacExistenceObservation,
    executable: Result<PathBuf, ()>,
}

impl Probe {
    fn active(identity: MacKernelIdentity, executable: Result<PathBuf, ()>) -> Self {
        Self {
            bsd: MacBsdObservation::Active(Ok(identity)),
            wait: MacWaitObservation::NotReapable,
            existence: MacExistenceObservation::Present,
            executable,
        }
    }

    fn zombie() -> Self {
        Self {
            bsd: MacBsdObservation::Zombie,
            wait: MacWaitObservation::NotReapable,
            existence: MacExistenceObservation::Present,
            executable: Err(()),
        }
    }

    fn unreadable_kernel() -> Self {
        Self {
            bsd: MacBsdObservation::Active(Err(())),
            wait: MacWaitObservation::NotReapable,
            existence: MacExistenceObservation::Present,
            executable: Err(()),
        }
    }

    fn fallback(wait: MacWaitObservation, existence: MacExistenceObservation) -> Self {
        Self {
            bsd: MacBsdObservation::Unavailable,
            wait,
            existence,
            executable: Err(()),
        }
    }
}

impl MacProcessProbe for Probe {
    fn bsd(&self, _pid: u32, _include_zombies: bool) -> MacBsdObservation {
        self.bsd
    }

    fn wait(&self, _pid: u32) -> MacWaitObservation {
        self.wait
    }

    fn existence(&self, _pid: u32) -> MacExistenceObservation {
        self.existence
    }

    fn executable(&self, _pid: u32) -> Result<PathBuf, ()> {
        self.executable.clone()
    }
}

fn identity() -> MacProcessIdentity {
    MacProcessIdentity {
        pid: PID,
        parent_pid: 7,
        started_at: 11,
        process_group: PID,
        executable: PathBuf::from(EXECUTABLE),
    }
}
