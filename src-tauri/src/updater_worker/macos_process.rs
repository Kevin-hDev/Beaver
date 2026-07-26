use std::path::Path;
use std::time::{Duration, Instant};
use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, System, UpdateKind};

use super::health::token_in_arguments;
use super::WorkerError;

const MAX_PROCESSES: usize = 4096;
const MAX_MATCHES: usize = 4;
const KILL_TIMEOUT: Duration = Duration::from_secs(5);

pub(crate) fn terminate_matching(executable: &Path, token: &str) -> Result<(), WorkerError> {
    let expected = std::fs::canonicalize(executable).map_err(|_| WorkerError)?;
    let mut system = System::new();
    refresh_all(&mut system);
    if system.processes().len() > MAX_PROCESSES {
        return Err(WorkerError);
    }
    let mut matches = Vec::with_capacity(1);
    for (pid, process) in system.processes() {
        if process.cmd().len() > 64 {
            continue;
        }
        let same_executable = process
            .exe()
            .and_then(|path| std::fs::canonicalize(path).ok())
            .is_some_and(|path| path == expected);
        if same_executable && token_in_arguments(process.cmd(), token) {
            if matches.len() == MAX_MATCHES {
                return Err(WorkerError);
            }
            matches.push((*pid, process.start_time()));
        }
    }
    for (pid, _) in &matches {
        if !system.process(*pid).is_some_and(sysinfo::Process::kill) {
            return Err(WorkerError);
        }
    }
    wait_for_exit(&mut system, &matches)
}

fn wait_for_exit(system: &mut System, matches: &[(sysinfo::Pid, u64)]) -> Result<(), WorkerError> {
    let started = Instant::now();
    loop {
        let mut remaining = false;
        for (pid, start_time) in matches {
            system.refresh_processes(ProcessesToUpdate::Some(&[*pid]), true);
            if system
                .process(*pid)
                .is_some_and(|process| process.start_time() == *start_time)
            {
                remaining = true;
            }
        }
        if !remaining {
            return Ok(());
        }
        if started.elapsed() >= KILL_TIMEOUT {
            return Err(WorkerError);
        }
        std::thread::sleep(Duration::from_millis(50));
    }
}

fn refresh_all(system: &mut System) {
    system.refresh_processes_specifics(
        ProcessesToUpdate::All,
        true,
        ProcessRefreshKind::nothing()
            .with_cmd(UpdateKind::Always)
            .with_exe(UpdateKind::Always),
    );
}
