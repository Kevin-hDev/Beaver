use std::io;
use std::sync::{Arc, Mutex, OnceLock};
use tauri::Manager;

// Tauri n'ignore prevent_exit que pour son code réservé : Beaver emploie donc
// une sentinelle distincte jusqu'à la fin du nettoyage coordonné.
const BEAVER_RESTART_REQUEST_CODE: i32 = i32::MAX - 1;

mod blocking;
mod cleanup;
mod emergency;
mod emergency_drain;
mod final_action;
mod policy;
mod presentation;
mod raw_exit;
mod registry;
mod registry_admission;
mod request_flow;
mod state;
#[cfg(test)]
mod test_api;
mod ultimate;
mod watchdog;
mod work_supervisor;

pub use work_supervisor::AppWorkSupervisor;
pub type AppWorkAdmission = registry::TrackedAdmission;
pub type AppWorkAdmissionError = registry::AdmissionError;

#[cfg(test)]
mod cleanup_tests;
#[cfg(test)]
mod coordinator_tests;
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
    ultimate: ultimate::UltimateExit,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ExitIntent {
    Exit,
    Restart,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum BeginResult {
    Started(policy::ShutdownTimeline, ExitIntent),
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
            ultimate: ultimate::UltimateExit::initialize()?,
        })
    }

    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "service producers adopt the app work supervisor during milestone 2"
        )
    )]
    pub(crate) fn work_supervisor(&self) -> AppWorkSupervisor {
        AppWorkSupervisor::new(self.registry.clone())
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
        let origin = std::time::Instant::now();
        if !self.registry.close() {
            return BeginResult::InvariantViolation;
        }
        if self.state.begin_closing() != state::BeginClosing::Started {
            return BeginResult::InvariantViolation;
        }
        let timeline = policy::ShutdownTimeline::from_origin(origin, self.policy);
        if self.intent.set(intent).is_err()
            || self.timeline.set(timeline).is_err()
            || !self.ultimate.arm(timeline.ultimate_deadline(), exit_code)
        {
            return BeginResult::InvariantViolation;
        }
        if close_cef(
            timeline.cef_admission_deadline(),
            timeline.cef_helper_exit_deadline(),
        ) == crate::services::browser::CefShutdownBarrier::TimedOut
        {
            ::log::warn!("[exit] CEF admission barrier exceeded; cleanup continues");
        }
        BeginResult::Started(timeline, intent)
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

pub fn request(app: &tauri::AppHandle, code: i32) {
    app.exit(code);
}

pub fn request_restart(app: &tauri::AppHandle) {
    request_restart_with(|code| app.exit(code));
}

fn request_restart_with(exit: impl FnOnce(i32)) {
    exit(BEAVER_RESTART_REQUEST_CODE);
}

pub fn handle_requested(app: &tauri::AppHandle, code: Option<i32>, api: &tauri::ExitRequestApi) {
    request_flow::handle_requested(app, code, api);
}

pub(crate) fn post_event_loop(app: &tauri::AppHandle) {
    ::log::info!("[exit] event loop returned");
    if let Some(coordinator) = app.try_state::<AppExitCoordinator>() {
        coordinator.drain_post_loop();
    }
}
