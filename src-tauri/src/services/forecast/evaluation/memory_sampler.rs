use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::Duration;
use sysinfo::{Pid, System};

const SAMPLE_INTERVAL: Duration = Duration::from_millis(100);
const MAX_TRACKED_PROCESSES: usize = 256;
const MAX_PROCESS_DEPTH: usize = 10;

pub(super) struct MemorySampler {
    stop: Arc<AtomicBool>,
    handle: Option<JoinHandle<u64>>,
}

impl MemorySampler {
    pub(super) fn start(root_pid: u32) -> Option<Self> {
        if root_pid < 2 {
            return None;
        }
        let stop = Arc::new(AtomicBool::new(false));
        let worker_stop = stop.clone();
        let handle = std::thread::Builder::new()
            .name("forecast-memory-sampler".into())
            .spawn(move || sample_until_stopped(root_pid, &worker_stop))
            .ok()?;
        Some(Self {
            stop,
            handle: Some(handle),
        })
    }

    pub(super) fn finish(mut self) -> Option<u64> {
        self.stop.store(true, Ordering::Release);
        let bytes = self.handle.take()?.join().ok()?;
        (bytes > 0).then(|| bytes_to_mb(bytes))
    }
}

impl Drop for MemorySampler {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
    }
}

fn sample_until_stopped(root_pid: u32, stop: &AtomicBool) -> u64 {
    let mut maximum = 0;
    let mut system = System::new();
    loop {
        system.refresh_processes(sysinfo::ProcessesToUpdate::All, true);
        maximum = maximum.max(sample_process_tree(&system, root_pid));
        if stop.load(Ordering::Acquire) {
            return maximum;
        }
        std::thread::sleep(SAMPLE_INTERVAL);
    }
}

fn sample_process_tree(system: &System, root_pid: u32) -> u64 {
    let mut tracked = Vec::with_capacity(16);
    tracked.push((Pid::from_u32(root_pid), 0_usize));
    let mut cursor = 0;
    let mut bytes = 0_u64;
    while cursor < tracked.len() && tracked.len() <= MAX_TRACKED_PROCESSES {
        let (pid, depth) = tracked[cursor];
        cursor += 1;
        if let Some(process) = system.process(pid) {
            bytes = bytes.saturating_add(process.memory());
        }
        if depth >= MAX_PROCESS_DEPTH {
            continue;
        }
        for (child_pid, process) in system.processes() {
            if tracked.len() >= MAX_TRACKED_PROCESSES {
                break;
            }
            if process.parent() == Some(pid)
                && !tracked.iter().any(|(known_pid, _)| known_pid == child_pid)
            {
                tracked.push((*child_pid, depth + 1));
            }
        }
    }
    bytes
}

fn bytes_to_mb(bytes: u64) -> u64 {
    bytes.div_ceil(1_048_576)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn memory_is_reported_in_rounded_megabytes() {
        assert_eq!(bytes_to_mb(1), 1);
        assert_eq!(bytes_to_mb(1_048_576), 1);
        assert_eq!(bytes_to_mb(1_048_577), 2);
    }
}
