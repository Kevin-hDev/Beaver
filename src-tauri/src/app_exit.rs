use std::io;
use std::sync::{Arc, Mutex, OnceLock};
use tauri::Manager;

mod blocking;
mod cleanup;
mod emergency;
mod emergency_drain;
mod policy;
mod presentation;
mod raw_exit;
mod registry;
mod registry_admission;
mod request_flow;
mod state;
mod ultimate;
mod watchdog;

#[cfg(test)]
mod cleanup_tests;
#[cfg(test)]
mod coordinator_tests;
#[cfg(test)]
mod emergency_tests;
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

    fn mark_ready(&self) -> bool {
        self.state.mark_ready()
    }

    fn spawn_watchdog(
        &self,
        app: tauri::AppHandle,
        timeline: policy::ShutdownTimeline,
        exit_code: i32,
    ) -> io::Result<()> {
        let actions = watchdog::WatchdogActions::production(move |code| app.exit(code));
        watchdog::WatchdogThread::spawn(
            timeline,
            Arc::clone(&self.state),
            self.emergency.clone(),
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

    #[cfg(test)]
    fn from_parts_for_test(
        policy: policy::ShutdownPolicy,
        ultimate: ultimate::UltimateExit,
    ) -> Self {
        Self {
            begin_lock: Mutex::new(()),
            state: Arc::new(state::ShutdownState::new()),
            registry: registry::AdmissionRegistry::new(),
            emergency: emergency::EmergencyInventory::new(),
            policy,
            timeline: OnceLock::new(),
            intent: OnceLock::new(),
            ultimate,
        }
    }

    #[cfg(test)]
    fn admit_for_test(&self) -> Result<registry::TrackedAdmission, registry::AdmissionError> {
        self.registry.try_admit()
    }

    #[cfg(test)]
    fn ultimate_is_armed_for_test(&self) -> bool {
        self.ultimate.is_armed_for_test()
    }

    #[cfg(test)]
    fn phase_for_test(&self) -> state::ShutdownPhase {
        self.state.phase()
    }

    #[cfg(test)]
    fn close_registry_for_test(&self) {
        assert!(self.registry.close());
    }

    #[cfg(test)]
    fn intent_for_test(&self) -> Option<ExitIntent> {
        self.intent.get().copied()
    }
}

pub fn request(app: &tauri::AppHandle, code: i32) {
    app.exit(code);
}

pub fn request_restart(app: &tauri::AppHandle) {
    app.exit(tauri::RESTART_EXIT_CODE);
}

pub fn handle_requested(app: &tauri::AppHandle, code: Option<i32>, api: &tauri::ExitRequestApi) {
    request_flow::handle_requested(app, code, api);
}

pub(crate) fn post_event_loop(app: &tauri::AppHandle) {
    if let Some(coordinator) = app.try_state::<AppExitCoordinator>() {
        coordinator.drain_post_loop();
    }
}
