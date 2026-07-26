use std::collections::VecDeque;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock};
use tokio::process::Child;
use tokio::task::AbortHandle;

const MAX_BACKGROUND_PROCESSES: usize = 16;

static NEXT_PROCESS_ID: AtomicU64 = AtomicU64::new(1);
static BACKGROUND_PROCESSES: OnceLock<Mutex<VecDeque<BackgroundProcess>>> = OnceLock::new();

struct BackgroundProcess {
    id: u64,
    abort: AbortHandle,
}

pub fn register_background_process(mut child: Child) -> (u64, Option<u32>) {
    let id = NEXT_PROCESS_ID.fetch_add(1, Ordering::Relaxed);
    let pid = child.id();
    let handle = tokio::spawn(async move {
        let _ = child.wait().await;
        unregister_background_process(id);
    });
    let abort = handle.abort_handle();
    drop(handle);

    let mut processes = registry().lock().unwrap();
    processes.push_back(BackgroundProcess { id, abort });
    while processes.len() > MAX_BACKGROUND_PROCESSES {
        if let Some(old) = processes.pop_front() {
            old.abort.abort();
        }
    }
    (id, pid)
}

fn unregister_background_process(id: u64) {
    let mut processes = registry().lock().unwrap();
    processes.retain(|process| process.id != id);
}

fn registry() -> &'static Mutex<VecDeque<BackgroundProcess>> {
    BACKGROUND_PROCESSES.get_or_init(|| Mutex::new(VecDeque::new()))
}

#[cfg(all(test, not(target_os = "windows")))]
pub(crate) fn abort_all_for_test() {
    let mut processes = registry().lock().unwrap();
    for process in processes.drain(..) {
        process.abort.abort();
    }
}
