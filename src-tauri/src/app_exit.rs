use std::io;
use std::sync::{Arc, Mutex, OnceLock};
use tauri::Manager;

// Tauri n'ignore prevent_exit que pour son code réservé : Beaver emploie donc
// une sentinelle distincte jusqu'à la fin du nettoyage coordonné.
const BEAVER_RESTART_REQUEST_CODE: i32 = i32::MAX - 1;

#[cfg(test)]
mod blocking;
mod cleanup;
mod coordinator_emergency;
mod emergency;
mod emergency_drain;
#[allow(dead_code)]
mod emergency_registration;
mod emergency_signaler;
mod final_action;
mod policy;
mod prearm;
pub(crate) use policy::OLLAMA_REAP_RESERVE_TIMEOUT;
mod presentation;
mod raw_exit;
mod registry;
mod registry_admission;
mod request_api;
mod request_flow;
mod state;
#[cfg(test)]
mod test_api;
mod ultimate;
mod watchdog;
mod work_supervisor;

#[cfg(all(test, unix))]
pub(crate) use emergency::EMERGENCY_CAPACITY;
#[allow(unused_imports)]
pub(crate) use emergency_registration::EmergencyHandoffReason;
#[allow(unused_imports)]
pub(crate) use emergency_signaler::{AppEmergencyPublisher, AppEmergencyRegistration};
pub use request_api::{request, request_restart};
pub use work_supervisor::AppWorkSupervisor;
pub type AppWorkAdmission = registry::TrackedAdmission;
pub type AppWorkAdmissionError = registry::AdmissionError;

#[cfg(test)]
mod cleanup_tests;
#[cfg(test)]
mod coordinator_tests;
#[cfg(test)]
mod emergency_signaler_tests;
#[cfg(test)]
mod emergency_tests;
#[cfg(test)]
mod final_action_tests;
#[cfg(test)]
mod policy_tests;
#[cfg(test)]
mod registry_tests;
#[cfg(test)]
mod state_tests;
#[cfg(test)]
mod ultimate_tests;
#[cfg(test)]
mod watchdog_tests;
#[cfg(test)]
mod work_supervisor_tests;

pub struct AppExitCoordinator {
    begin_lock: Mutex<()>,
    state: Arc<state::ShutdownState>,
    registry: registry::AdmissionRegistry,
    emergency: emergency::EmergencyInventory,
    policy: policy::ShutdownPolicy,
    timeline: OnceLock<policy::ShutdownTimeline>,
    intent: OnceLock<ExitIntent>,
    exit_code: OnceLock<i32>,
    ultimate: ultimate::UltimateExit,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ExitIntent {
    Exit,
    Restart,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum BeginResult {
    Started(policy::ShutdownTimeline, ExitIntent, i32),
    Waiting,
    Ready,
    InvariantViolation,
}

impl AppExitCoordinator {
    pub fn initialize() -> io::Result<Self> {
        Ok(Self {
            begin_lock: Mutex::new(()),
            state: Arc::new(state::ShutdownState::new()),
            registry: registry::AdmissionRegistry::new(),
            emergency: emergency::EmergencyInventory::new(),
            policy: policy::ShutdownPolicy::production(),
            timeline: OnceLock::new(),
            intent: OnceLock::new(),
            exit_code: OnceLock::new(),
            ultimate: ultimate::UltimateExit::initialize()?,
        })
    }

    pub(crate) fn work_supervisor(&self) -> AppWorkSupervisor {
        AppWorkSupervisor::new(self.registry.clone())
    }

    #[cfg(test)]
    pub(crate) fn close_work_admission_for_test(&self) {
        let _ = self.registry.close();
    }

    #[cfg(test)]
    fn begin(&self, exit_code: i32) -> BeginResult {
        self.begin_with_intent(
            ExitIntent::Exit,
            exit_code,
            crate::services::browser::begin_cef_shutdown,
        )
    }

    #[cfg(test)]
    fn begin_with_cef_close(
        &self,
        exit_code: i32,
        close_cef: impl FnOnce(
            std::time::Instant,
            std::time::Instant,
            std::time::Instant,
        ) -> crate::services::browser::CefShutdownBarrier,
    ) -> BeginResult {
        self.begin_with_intent(ExitIntent::Exit, exit_code, close_cef)
    }

    fn begin_with_intent(
        &self,
        intent: ExitIntent,
        exit_code: i32,
        close_cef: impl FnOnce(
            std::time::Instant,
            std::time::Instant,
            std::time::Instant,
        ) -> crate::services::browser::CefShutdownBarrier,
    ) -> BeginResult {
        let _guard = match self.begin_lock.lock() {
            Ok(guard) => guard,
            Err(_) => return BeginResult::InvariantViolation,
        };
        match self.state.phase() {
            state::ShutdownPhase::ReadyToExit => return BeginResult::Ready,
            state::ShutdownPhase::Closing => return BeginResult::Waiting,
            state::ShutdownPhase::Running => {}
        }
        if !self.registry.close() {
            return BeginResult::InvariantViolation;
        }
        if self.state.begin_closing() != state::BeginClosing::Started {
            return BeginResult::InvariantViolation;
        }
        let Some((timeline, owned_intent, owned_exit_code)) =
            self.prepare_exit_locked(intent, exit_code)
        else {
            return BeginResult::InvariantViolation;
        };
        if close_cef(
            timeline.cef_admission_deadline(),
            timeline.cef_helper_exit_deadline(),
            timeline.ultimate_deadline(),
        ) == crate::services::browser::CefShutdownBarrier::TimedOut
        {
            ::log::warn!("[exit] CEF admission barrier exceeded; cleanup continues");
        }
        BeginResult::Started(timeline, owned_intent, owned_exit_code)
    }

    fn spawn_watchdog(
        &self,
        app: tauri::AppHandle,
        timeline: policy::ShutdownTimeline,
        intent: ExitIntent,
        exit_code: i32,
    ) -> io::Result<()> {
        let actions = watchdog::WatchdogActions::production(move |intent, code| {
            final_action::dispatch_tauri(&app, intent, code);
        });
        watchdog::WatchdogThread::spawn(
            timeline,
            Arc::clone(&self.state),
            self.emergency.clone(),
            intent,
            exit_code,
            actions,
        )
        .map(drop)
    }

    fn drain_post_loop(&self) {
        if let Some(timeline) = self.timeline.get().copied() {
            watchdog::drain_post_loop(&self.emergency, timeline);
        }
    }
}

pub fn handle_requested(app: &tauri::AppHandle, code: Option<i32>, api: &tauri::ExitRequestApi) {
    request_flow::handle_requested(app, code, api);
}

pub(crate) fn post_event_loop(app: &tauri::AppHandle) {
    ::log::info!("[exit] event loop returned");
    let webviews = crate::services::browser::observe_native_webviews();
    ::log::info!(
        "[exit] native WebView descendants={} shared-system={}",
        webviews.dedicated_pids.len(),
        webviews.shared_system_count
    );
    if let Some(coordinator) = app.try_state::<AppExitCoordinator>() {
        coordinator.drain_post_loop();
    }
}
