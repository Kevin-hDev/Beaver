use super::super::constants::CEF_HELPER_WAIT_SLICE;
use super::super::CefUnavailableCategory;
use super::clock;
use super::objects::WindowsHelperObjects;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::JoinHandle;
#[cfg(not(feature = "windows-tests"))]
use windows_sys::Win32::System::Threading::{GetCurrentProcess, TerminateProcess};

type TerminationAction = Arc<dyn Fn() + Send + Sync>;

#[derive(Debug)]
pub(in crate::services::browser) struct WindowsHelperMonitor {
    stop: Arc<AtomicBool>,
    worker: Option<JoinHandle<()>>,
}

impl WindowsHelperMonitor {
    pub(in crate::services::browser) fn start(
        objects: Arc<WindowsHelperObjects>,
        generation: u64,
    ) -> Result<Self, CefUnavailableCategory> {
        #[cfg(not(feature = "windows-tests"))]
        let action = Arc::new(terminate_current_process) as TerminationAction;
        #[cfg(feature = "windows-tests")]
        let action = Arc::new(|| {}) as TerminationAction;
        Self::start_with_shared_action(objects, generation, action)
    }

    #[cfg(test)]
    pub(super) fn start_with_action(
        objects: Arc<WindowsHelperObjects>,
        generation: u64,
        action: impl Fn() + Send + Sync + 'static,
    ) -> Result<Self, CefUnavailableCategory> {
        Self::start_with_shared_action(objects, generation, Arc::new(action))
    }

    fn start_with_shared_action(
        objects: Arc<WindowsHelperObjects>,
        generation: u64,
        action: TerminationAction,
    ) -> Result<Self, CefUnavailableCategory> {
        let stop = Arc::new(AtomicBool::new(false));
        let worker_stop = Arc::clone(&stop);
        let worker = std::thread::Builder::new()
            .name("beaver-cef-helper-monitor".to_string())
            .spawn(move || monitor_loop(objects, generation, worker_stop, action))
            .map_err(|_| CefUnavailableCategory::Reaper)?;
        Ok(Self {
            stop,
            worker: Some(worker),
        })
    }
}

impl Drop for WindowsHelperMonitor {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        if let Some(worker) = self.worker.take() {
            worker.thread().unpark();
            if worker.join().is_err() {
                std::process::abort();
            }
        }
    }
}

fn monitor_loop(
    objects: Arc<WindowsHelperObjects>,
    generation: u64,
    stop: Arc<AtomicBool>,
    action: TerminationAction,
) {
    while !stop.load(Ordering::Acquire) {
        let control = match objects.control_snapshot() {
            Ok(control) if control.generation == generation => control,
            _ => return action(),
        };
        if control.closing {
            match clock::reached(control.deadline_ticks) {
                Ok(false) => std::thread::park_timeout(CEF_HELPER_WAIT_SLICE),
                _ => return action(),
            }
        } else {
            match objects.wait_for_closing(wait_millis()) {
                Ok(_) => {}
                Err(_) => return action(),
            }
        }
    }
}

fn wait_millis() -> u32 {
    CEF_HELPER_WAIT_SLICE
        .as_millis()
        .clamp(1, u128::from(u32::MAX)) as u32
}

#[cfg(not(feature = "windows-tests"))]
fn terminate_current_process() {
    if unsafe { TerminateProcess(GetCurrentProcess(), 1) } == 0 {
        std::process::abort();
    }
}
