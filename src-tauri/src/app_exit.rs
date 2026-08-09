use std::io;
use std::sync::{Arc, OnceLock};
use tauri::{ExitRequestApi, Manager};

mod blocking;
mod cleanup;
mod emergency;
mod emergency_drain;
mod policy;
mod raw_exit;
mod registry;
mod registry_admission;
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
    state: Arc<state::ShutdownState>,
    registry: registry::AdmissionRegistry,
    emergency: emergency::EmergencyInventory,
    policy: policy::ShutdownPolicy,
    timeline: OnceLock<policy::ShutdownTimeline>,
    ultimate: ultimate::UltimateExit,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum BeginResult {
    Started(policy::ShutdownTimeline),
    Waiting,
    Ready,
}

impl AppExitCoordinator {
    pub fn initialize() -> io::Result<Self> {
        Ok(Self {
            state: Arc::new(state::ShutdownState::new()),
            registry: registry::AdmissionRegistry::new(),
            emergency: emergency::EmergencyInventory::new(),
            policy: policy::ShutdownPolicy::production(),
            timeline: OnceLock::new(),
            ultimate: ultimate::UltimateExit::initialize()?,
        })
    }

    fn begin(&self, exit_code: i32) -> BeginResult {
        let origin = std::time::Instant::now();
        if !self.registry.close() {
            return match self.state.phase() {
                state::ShutdownPhase::ReadyToExit => BeginResult::Ready,
                _ => BeginResult::Waiting,
            };
        }
        if self.state.begin_closing() != state::BeginClosing::Started {
            raw_exit::terminate_process(1);
        }
        let timeline = policy::ShutdownTimeline::from_origin(origin, self.policy);
        if self.timeline.set(timeline).is_err()
            || !self.ultimate.arm(timeline.ultimate_deadline(), exit_code)
        {
            raw_exit::terminate_process(1);
        }
        BeginResult::Started(timeline)
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
            state: Arc::new(state::ShutdownState::new()),
            registry: registry::AdmissionRegistry::new(),
            emergency: emergency::EmergencyInventory::new(),
            policy,
            timeline: OnceLock::new(),
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
}

pub fn request(app: &tauri::AppHandle, code: i32) {
    app.exit(code);
}

pub fn handle_requested(app: &tauri::AppHandle, code: Option<i32>, api: &ExitRequestApi) {
    if code == Some(tauri::RESTART_EXIT_CODE) {
        return;
    }
    let exit_code = code.unwrap_or_default();
    let coordinator = app.state::<AppExitCoordinator>();
    match coordinator.begin(exit_code) {
        BeginResult::Ready => {}
        BeginResult::Waiting => api.prevent_exit(),
        BeginResult::Started(timeline) => {
            api.prevent_exit();
            if coordinator
                .spawn_watchdog(app.clone(), timeline, exit_code)
                .is_err()
            {
                ::log::error!("[exit] watchdog unavailable; ultimate guard remains armed");
            }
            hide_application(app);
            let handle = app.clone();
            let registry = coordinator.registry.clone();
            tauri::async_runtime::spawn(async move {
                let started = std::time::Instant::now();
                if !registry
                    .wait_empty_until(timeline.graceful_deadline())
                    .await
                {
                    ::log::warn!("[exit] tracked work exceeded graceful deadline");
                }
                match cleanup::run(&handle, timeline).await {
                    cleanup::CleanupOutcome::Completed => {}
                    cleanup::CleanupOutcome::TimedOut => {
                        ::log::warn!("[exit] graceful deadline reached")
                    }
                    cleanup::CleanupOutcome::Panicked => {
                        ::log::warn!("[exit] cleanup interrupted")
                    }
                }
                ::log::info!("[exit] cleanup completed in {:?}", started.elapsed());
                if handle.state::<AppExitCoordinator>().mark_ready() {
                    handle.exit(exit_code);
                }
            });
        }
    }
}

pub(crate) fn post_event_loop(app: &tauri::AppHandle) {
    if let Some(coordinator) = app.try_state::<AppExitCoordinator>() {
        coordinator.drain_post_loop();
    }
}

fn hide_application(app: &tauri::AppHandle) {
    for label in ["main", "mascot"] {
        if let Some(window) = app.get_webview_window(label) {
            let _ = window.hide();
        }
    }
    #[cfg(target_os = "macos")]
    let _ = app.set_dock_visibility(false);
}
