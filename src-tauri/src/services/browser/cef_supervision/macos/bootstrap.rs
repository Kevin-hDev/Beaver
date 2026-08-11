use super::super::constants::{CEF_ADMISSION_TIMEOUT, CEF_HELPER_WAIT_SLICE};
use super::super::{CefIpcNames, CefLaunchMarker, CefUnavailableCategory};
use super::identity::MacProcessIdentity;
use super::MacHelperObjects;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

pub(in crate::services::browser) struct MacHelperBootstrap {
    objects: Arc<MacHelperObjects>,
    generation: u64,
    parent_pid: u32,
    identity: MacProcessIdentity,
}

pub(in crate::services::browser) struct MacHelperAdmission {
    stop_monitor: Arc<AtomicBool>,
    monitor: Option<std::thread::JoinHandle<()>>,
    _objects: Arc<MacHelperObjects>,
}

impl MacHelperBootstrap {
    pub(in crate::services::browser) fn prepare(
        encoded_marker: &str,
        root: &Path,
    ) -> Result<Self, CefUnavailableCategory> {
        let marker = CefLaunchMarker::decode_unique(&[encoded_marker])
            .map_err(|_| CefUnavailableCategory::Admission)?;
        let names =
            CefIpcNames::from_marker(&marker).map_err(|_| CefUnavailableCategory::Object)?;
        if unsafe { libc::setpgid(0, 0) } != 0 {
            return Err(CefUnavailableCategory::Admission);
        }
        let identity = MacProcessIdentity::read(std::process::id())?;
        if identity.process_group != identity.pid {
            return Err(CefUnavailableCategory::Admission);
        }
        let parent_pid = MacProcessIdentity::read(identity.parent_pid)?.pid;
        Ok(Self {
            objects: Arc::new(MacHelperObjects::open(root, &names)?),
            generation: marker.generation(),
            parent_pid,
            identity,
        })
    }

    pub(in crate::services::browser) fn admit_after_sandbox(
        self,
    ) -> Result<MacHelperAdmission, CefUnavailableCategory> {
        self.objects
            .publish(
                self.generation,
                self.identity.pid,
                self.identity.started_at,
                self.identity.process_group,
            )
            .map_err(|_| CefUnavailableCategory::Admission)?;
        let stop_monitor = Arc::new(AtomicBool::new(false));
        let monitor = start_monitor(
            Arc::clone(&self.objects),
            Arc::clone(&stop_monitor),
            self.parent_pid,
            self.generation,
        )?;
        wait_for_admission(&self.objects, self.generation)?;
        Ok(MacHelperAdmission {
            stop_monitor,
            monitor: Some(monitor),
            _objects: self.objects,
        })
    }
}

impl Drop for MacHelperAdmission {
    fn drop(&mut self) {
        self.stop_monitor.store(true, Ordering::Release);
        if let Some(monitor) = self.monitor.take() {
            let _ = monitor.join();
        }
    }
}

fn wait_for_admission(
    objects: &MacHelperObjects,
    generation: u64,
) -> Result<(), CefUnavailableCategory> {
    let deadline = Instant::now() + CEF_ADMISSION_TIMEOUT;
    loop {
        let control = objects
            .control_snapshot()
            .map_err(|_| CefUnavailableCategory::Admission)?;
        if control.closing || control.generation != generation || closing_or_failure(objects) {
            return Err(CefUnavailableCategory::Admission);
        }
        if objects
            .admission_signaled()
            .map_err(|_| CefUnavailableCategory::Admission)?
        {
            return Ok(());
        }
        if Instant::now() >= deadline {
            return Err(CefUnavailableCategory::Admission);
        }
        std::thread::sleep(CEF_HELPER_WAIT_SLICE);
    }
}

fn start_monitor(
    objects: Arc<MacHelperObjects>,
    stop: Arc<AtomicBool>,
    parent_pid: u32,
    generation: u64,
) -> Result<std::thread::JoinHandle<()>, CefUnavailableCategory> {
    std::thread::Builder::new()
        .name("cef-helper-monitor".to_string())
        .spawn(move || loop {
            if stop.load(Ordering::Acquire) {
                return;
            }
            let parent_gone = parent_changed(parent_pid);
            let close = control_requires_exit(&objects, generation);
            if parent_gone || close {
                unsafe { libc::_exit(1) };
            }
            std::thread::sleep(Duration::from_millis(10));
        })
        .map_err(|_| CefUnavailableCategory::Reaper)
}

pub(super) fn parent_changed(expected: u32) -> bool {
    // Seatbelt peut refuser l'inspection d'un autre processus. `getppid`
    // relit uniquement la relation parentale du helper lui-même.
    let current = unsafe { libc::getppid() };
    current <= 0 || current as u32 != expected
}

fn closing_or_failure(objects: &MacHelperObjects) -> bool {
    objects.closing_signaled() != Ok(false)
}

fn control_requires_exit(objects: &MacHelperObjects, generation: u64) -> bool {
    match objects.control_snapshot() {
        Ok(state) if state.generation != generation => true,
        Ok(state) if !state.closing => false,
        Ok(state) => super::clock::reached(state.deadline_ticks).unwrap_or(true),
        Err(_) => true,
    }
}
