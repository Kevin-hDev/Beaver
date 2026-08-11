use super::state::ShutdownState;
use super::ExitIntent;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum FinalActionSource {
    Cleanup,
    Watchdog,
}

pub(super) fn run(
    state: &ShutdownState,
    intent: ExitIntent,
    exit_code: i32,
    source: FinalActionSource,
    dispatch: impl FnOnce(ExitIntent, i32),
) -> bool {
    // Cette transition est l'autorité unique afin que deux chemins concurrents
    // ne puissent jamais déclencher deux actions finales.
    if !state.mark_ready() {
        ::log::info!("[exit] final action already claimed");
        return false;
    }
    if source == FinalActionSource::Watchdog && intent == ExitIntent::Restart {
        ::log::warn!("[exit] restart triggered by watchdog");
    }
    dispatch(intent, exit_code);
    true
}

pub(super) fn dispatch_tauri(app: &tauri::AppHandle, intent: ExitIntent, exit_code: i32) {
    match intent {
        ExitIntent::Exit => app.exit(exit_code),
        ExitIntent::Restart => app.request_restart(),
    }
}
