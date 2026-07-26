use std::time::{Duration, Instant};
use sysinfo::{Pid, ProcessesToUpdate, System};

use super::WorkerError;

const POLL_INTERVAL: Duration = Duration::from_millis(100);

pub(crate) fn wait_for_parent(pid: u32, timeout: Duration) -> Result<(), WorkerError> {
    if pid == 0 || timeout.is_zero() {
        return Err(WorkerError);
    }
    let pid = Pid::from_u32(pid);
    let mut system = System::new();
    refresh(&mut system, pid);
    let Some(start_time) = system.process(pid).map(|process| process.start_time()) else {
        return Ok(());
    };
    let started = Instant::now();
    loop {
        refresh(&mut system, pid);
        match system.process(pid) {
            None => return Ok(()),
            Some(process) if process.start_time() != start_time => return Ok(()),
            Some(_) if started.elapsed() >= timeout => return Err(WorkerError),
            Some(_) => std::thread::sleep(POLL_INTERVAL.min(timeout)),
        }
    }
}

fn refresh(system: &mut System, pid: Pid) {
    system.refresh_processes(ProcessesToUpdate::Some(&[pid]), true);
}

#[cfg(test)]
#[path = "process_tests.rs"]
mod tests;
