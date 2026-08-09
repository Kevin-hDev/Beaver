use std::io;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::atomic::{AtomicI32, AtomicU64, AtomicU8, Ordering};
use std::sync::Arc;
use std::thread::{JoinHandle, Thread};
use std::time::{Duration, Instant};

const PARKED: u8 = 0;
const CONFIGURING: u8 = 1;
const ARMED: u8 = 2;
const FIRED: u8 = 3;
const STOPPED: u8 = 4;
const DEFAULT_FAILURE_CODE: i32 = 1;

type ExitAction = Arc<dyn Fn(i32) + Send + Sync + 'static>;

pub(super) struct RawExitActions {
    primary: ExitAction,
    fallback: ExitAction,
}

impl RawExitActions {
    fn production() -> Self {
        Self {
            primary: Arc::new(|code| super::raw_exit::terminate_process(code)),
            fallback: Arc::new(|code| super::raw_exit::terminate_process(code)),
        }
    }

    #[cfg(test)]
    pub(super) fn testing(
        primary: impl Fn(i32) + Send + Sync + 'static,
        fallback: impl Fn(i32) + Send + Sync + 'static,
    ) -> Self {
        Self {
            primary: Arc::new(primary),
            fallback: Arc::new(fallback),
        }
    }
}

struct UltimateControl {
    origin: Instant,
    state: AtomicU8,
    deadline_nanos: AtomicU64,
    exit_code: AtomicI32,
    actions: RawExitActions,
}

pub(super) struct UltimateExit {
    control: Arc<UltimateControl>,
    thread: Thread,
    join: Option<JoinHandle<()>>,
}

impl UltimateExit {
    pub(super) fn initialize() -> io::Result<Self> {
        Self::initialize_inner(Instant::now(), RawExitActions::production(), |worker| {
            std::thread::Builder::new()
                .name("beaver-ultimate-exit".to_string())
                .spawn(worker)
        })
    }

    pub(super) fn arm(&self, deadline: Instant, code: i32) -> bool {
        if self
            .control
            .state
            .compare_exchange(PARKED, CONFIGURING, Ordering::AcqRel, Ordering::Acquire)
            .is_err()
        {
            return false;
        }
        let offset = deadline.saturating_duration_since(self.control.origin);
        self.control
            .deadline_nanos
            .store(duration_nanos(offset), Ordering::Relaxed);
        self.control.exit_code.store(code, Ordering::Relaxed);
        self.control.state.store(ARMED, Ordering::Release);
        self.thread.unpark();
        true
    }

    #[cfg(test)]
    pub(super) fn initialize_for_test(
        origin: Instant,
        actions: RawExitActions,
    ) -> io::Result<Self> {
        Self::initialize_with(origin, actions, |worker| {
            std::thread::Builder::new()
                .name("beaver-ultimate-exit-test".to_string())
                .spawn(worker)
        })
    }

    #[cfg(test)]
    pub(super) fn initialize_with<Spawn>(
        origin: Instant,
        actions: RawExitActions,
        spawn: Spawn,
    ) -> io::Result<Self>
    where
        Spawn: FnOnce(Box<dyn FnOnce() + Send + 'static>) -> io::Result<JoinHandle<()>>,
    {
        Self::initialize_inner(origin, actions, spawn)
    }

    fn initialize_inner<Spawn>(
        origin: Instant,
        actions: RawExitActions,
        spawn: Spawn,
    ) -> io::Result<Self>
    where
        Spawn: FnOnce(Box<dyn FnOnce() + Send + 'static>) -> io::Result<JoinHandle<()>>,
    {
        let control = Arc::new(UltimateControl {
            origin,
            state: AtomicU8::new(PARKED),
            deadline_nanos: AtomicU64::new(0),
            exit_code: AtomicI32::new(DEFAULT_FAILURE_CODE),
            actions,
        });
        let worker_control = Arc::clone(&control);
        let join = spawn(Box::new(move || worker_entry(worker_control)))?;
        let thread = join.thread().clone();
        Ok(Self {
            control,
            thread,
            join: Some(join),
        })
    }

    #[cfg(test)]
    pub(super) fn stop_for_test(&mut self) {
        self.stop_and_join();
    }

    #[cfg(test)]
    pub(super) fn is_armed_for_test(&self) -> bool {
        matches!(self.control.state.load(Ordering::Acquire), ARMED | FIRED)
    }

    fn stop_and_join(&mut self) {
        self.control.state.store(STOPPED, Ordering::Release);
        self.thread.unpark();
        if let Some(join) = self.join.take() {
            let _ = join.join();
        }
    }
}

impl Drop for UltimateExit {
    fn drop(&mut self) {
        self.stop_and_join();
    }
}

fn worker_entry(control: Arc<UltimateControl>) {
    let result = catch_unwind(AssertUnwindSafe(|| worker_loop(&control)));
    if result.is_err() {
        let code = control.exit_code.load(Ordering::Acquire);
        (control.actions.fallback)(code);
    }
}

fn worker_loop(control: &UltimateControl) {
    loop {
        match control.state.load(Ordering::Acquire) {
            PARKED | CONFIGURING => std::thread::park(),
            STOPPED | FIRED => return,
            ARMED => {
                let deadline = control.deadline_nanos.load(Ordering::Acquire);
                let now = duration_nanos(control.origin.elapsed());
                if now < deadline {
                    std::thread::park_timeout(Duration::from_nanos(deadline - now));
                    continue;
                }
                if control
                    .state
                    .compare_exchange(ARMED, FIRED, Ordering::AcqRel, Ordering::Acquire)
                    .is_ok()
                {
                    let code = control.exit_code.load(Ordering::Acquire);
                    (control.actions.primary)(code);
                    return;
                }
            }
            _ => return,
        }
    }
}

fn duration_nanos(duration: Duration) -> u64 {
    duration.as_nanos().min(u64::MAX as u128) as u64
}
