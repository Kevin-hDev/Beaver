#[cfg(target_os = "macos")]
use super::pump_gate::PumpGate;
#[cfg(target_os = "macos")]
use std::sync::Arc;
#[cfg(target_os = "macos")]
use std::sync::OnceLock;

#[derive(Clone)]
pub(super) struct PumpScheduler {
    app: tauri::AppHandle,
    #[cfg(target_os = "macos")]
    gate: Arc<PumpGate>,
    #[cfg(target_os = "macos")]
    wake: Arc<OnceLock<super::native_pump::PumpWake>>,
}

impl PumpScheduler {
    pub(super) fn new(app: tauri::AppHandle) -> Self {
        Self {
            app,
            #[cfg(target_os = "macos")]
            gate: Arc::new(PumpGate::default()),
            #[cfg(target_os = "macos")]
            wake: Arc::new(OnceLock::new()),
        }
    }

    pub(super) fn app(&self) -> &tauri::AppHandle {
        &self.app
    }

    pub(super) fn start_fallback(&self) -> Result<(), ()> {
        #[cfg(target_os = "macos")]
        {
            let wake = super::native_pump::start(self.gate.clone())?;
            self.wake.set(wake).map_err(|_| ())?;
            Ok(())
        }
        #[cfg(target_os = "windows")]
        Ok(())
    }

    pub(super) fn schedule(&self, requested_delay_ms: i64) {
        #[cfg(target_os = "macos")]
        {
            if let Some(wake) = self.wake.get() {
                wake.notify(requested_delay_ms);
            } else {
                let _ = self.gate.request();
            }
        }
        #[cfg(target_os = "windows")]
        let _ = requested_delay_ms;
    }

    pub(super) fn stop(&self) {
        #[cfg(target_os = "macos")]
        {
            self.gate.stop();
            super::native_pump::stop();
        }
    }
}

#[cfg(target_os = "macos")]
pub(super) const fn fallback_pump_interval_ms() -> u64 {
    33
}
