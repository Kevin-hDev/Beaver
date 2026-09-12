const WATCH_INTERVAL_MS: u64 = 100;

type Identity = crate::services::owned_process::OwnedProcessIdentity;

#[derive(Debug, PartialEq, Eq)]
pub(super) enum WatchdogAction {
    Wait,
    Exit,
    Kill,
}

pub(super) fn run(parent: Identity, root_pid: u32, root_start: u64) -> Result<i32, String> {
    loop {
        let parent_alive = inspect_parent(parent).map_err(|_| ());
        let root = inspect_root(root_pid, root_start).map_err(|_| ());
        match watchdog_action(parent_alive, root.as_ref().map(Option::is_some).map_err(|_| ())) {
            WatchdogAction::Exit => return Ok(0),
            WatchdogAction::Kill => {
                if let Ok(Some(current_root)) = root {
                    let _ = crate::services::owned_process::OwnedProcess::signal_exact(
                        current_root,
                        true,
                    );
                }
                return Ok(0);
            }
            WatchdogAction::Wait => {}
        }
        std::thread::sleep(std::time::Duration::from_millis(WATCH_INTERVAL_MS));
    }
}

fn inspect_parent(expected: Identity) -> Result<bool, String> {
    match crate::services::owned_process::OwnedProcess::identity(expected.pid) {
        Ok(current) => Ok(current == expected),
        Err(_) if !crate::services::owned_process::OwnedProcess::process_exists(expected.pid) => {
            Ok(false)
        }
        Err(_) => Err(error()),
    }
}

pub(super) fn inspect_root(pid: u32, start: u64) -> Result<Option<Identity>, String> {
    match crate::services::owned_process::OwnedProcess::inspect_for_recovery(pid, start) {
        Ok(crate::services::owned_process::OwnedProcessInspection::Owned(identity)) => {
            Ok(Some(identity))
        }
        Ok(crate::services::owned_process::OwnedProcessInspection::Unowned) => Ok(None),
        Err(_) if !crate::services::owned_process::OwnedProcess::process_exists(pid) => Ok(None),
        Err(_) => Err(error()),
    }
}

pub(super) fn watchdog_action(
    parent_alive: Result<bool, ()>,
    root_alive: Result<bool, ()>,
) -> WatchdogAction {
    match (parent_alive, root_alive) {
        (_, Ok(false)) => WatchdogAction::Exit,
        (Ok(false), Ok(true)) => WatchdogAction::Kill,
        _ => WatchdogAction::Wait,
    }
}

fn error() -> String {
    super::launch::sandbox_error()
}
