use super::super::constants::{CEF_ADMISSION_TIMEOUT, CEF_HELPER_WAIT_SLICE};
use super::super::{CefIpcNames, CefLaunchMarker, CefUnavailableCategory};
use super::identity::MacProcessIdentity;
use super::MacHelperObjects;
use std::os::unix::ffi::OsStrExt;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use zeroize::Zeroizing;

const ADMISSION_PREFIX: &str = "--beaver-cef-admission=";
const TYPE_PREFIX: &str = "--type=";
const MAX_ARGUMENTS: usize = 64;
const MAX_ARGUMENT_BYTES: usize = 2_048;

pub(in crate::services::browser) struct MacHelperBootstrap {
    objects: Arc<MacHelperObjects>,
    generation: u64,
    parent_identity: MacProcessIdentity,
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
        let parent_identity = MacProcessIdentity::read(identity.parent_pid)?;
        Ok(Self {
            objects: Arc::new(MacHelperObjects::open(root, &names)?),
            generation: marker.generation(),
            parent_identity,
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
            self.parent_identity,
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

pub(in crate::services::browser) fn parse_helper_marker(
) -> Result<Zeroizing<String>, CefUnavailableCategory> {
    let mut marker = None;
    let mut process_type = false;
    for (index, raw) in std::env::args_os().enumerate() {
        if index >= MAX_ARGUMENTS || raw.as_os_str().as_bytes().len() > MAX_ARGUMENT_BYTES {
            return Err(CefUnavailableCategory::Admission);
        }
        let value = Zeroizing::new(
            raw.into_string()
                .map_err(|_| CefUnavailableCategory::Admission)?,
        );
        if let Some(found) = strip_ascii_prefix(&value, ADMISSION_PREFIX) {
            if found.is_empty() || marker.is_some() {
                return Err(CefUnavailableCategory::Admission);
            }
            marker = Some(Zeroizing::new(found.to_string()));
        } else if let Some(found) = strip_ascii_prefix(&value, TYPE_PREFIX) {
            if found.is_empty() || process_type {
                return Err(CefUnavailableCategory::Admission);
            }
            process_type = true;
        }
    }
    match (process_type, marker) {
        (true, Some(marker)) => Ok(marker),
        _ => Err(CefUnavailableCategory::Admission),
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
    parent_identity: MacProcessIdentity,
    generation: u64,
) -> Result<std::thread::JoinHandle<()>, CefUnavailableCategory> {
    std::thread::Builder::new()
        .name("cef-helper-monitor".to_string())
        .spawn(move || loop {
            if stop.load(Ordering::Acquire) {
                return;
            }
            let parent_gone = match parent_identity.is_alive() {
                Ok(alive) => !alive,
                Err(_) => true,
            };
            let close = closing_or_failure(&objects) || control_requires_exit(&objects, generation);
            if parent_gone || close {
                unsafe { libc::_exit(1) };
            }
            std::thread::sleep(Duration::from_millis(10));
        })
        .map_err(|_| CefUnavailableCategory::Reaper)
}

fn closing_or_failure(objects: &MacHelperObjects) -> bool {
    objects.closing_signaled() != Ok(false)
}

fn control_requires_exit(objects: &MacHelperObjects, generation: u64) -> bool {
    match objects.control_snapshot() {
        Ok(state) => state.closing || state.generation != generation,
        Err(_) => true,
    }
}

fn strip_ascii_prefix<'a>(value: &'a str, prefix: &str) -> Option<&'a str> {
    value
        .get(..prefix.len())?
        .eq_ignore_ascii_case(prefix)
        .then(|| &value[prefix.len()..])
}
