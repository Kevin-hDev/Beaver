mod args;
mod command;
mod health;
mod linux;
mod macos;
mod macos_bundle;
mod macos_mount;
mod macos_process;
mod macos_swap;
mod process;
mod verify;
mod windows;

pub(crate) const UPDATE_TEMP_PREFIX: &str = "cl-go-dash-update";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Platform {
    Macos,
    Windows,
    Linux,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorkerError;

impl std::fmt::Display for WorkerError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("update failed")
    }
}

impl std::error::Error for WorkerError {}

pub fn run_from_env() -> Result<(), WorkerError> {
    let _cleanup = verify::SelfCleanup::prepare();
    let platform = current_platform();
    let arguments = args::parse_from(std::env::args_os(), &std::env::temp_dir(), platform)?;
    let current =
        verify::current_installation(&std::env::current_dir().map_err(|_| WorkerError)?, platform)?;
    process::wait_for_parent(arguments.parent_pid, std::time::Duration::from_secs(120))?;
    match platform {
        Platform::Macos => macos::apply(&arguments.asset, &current),
        Platform::Windows => windows::apply(&arguments.asset, &current),
        Platform::Linux => linux::apply(&arguments.asset, &current),
    }
}

fn current_platform() -> Platform {
    if cfg!(target_os = "macos") {
        Platform::Macos
    } else if cfg!(target_os = "windows") {
        Platform::Windows
    } else {
        Platform::Linux
    }
}
