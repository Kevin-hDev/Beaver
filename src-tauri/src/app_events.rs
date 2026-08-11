use crate::models::ClgoConfig;
use std::ffi::OsStr;
use tauri::{Manager, RunEvent, WindowEvent};

pub const AUTOSTART_ARG: &str = "--clgo-autostart";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum MainWindowCloseAction {
    Hide,
    Quit,
}

const fn main_window_close_action(is_macos: bool) -> MainWindowCloseAction {
    if is_macos {
        MainWindowCloseAction::Hide
    } else {
        MainWindowCloseAction::Quit
    }
}

pub fn handle_run_event(app_handle: &tauri::AppHandle, event: RunEvent) {
    report_run_event(&event);
    match event {
        RunEvent::WindowEvent {
            label,
            event: WindowEvent::CloseRequested { api, .. },
            ..
        } => {
            if label == "main" {
                api.prevent_close();
                match main_window_close_action(cfg!(target_os = "macos")) {
                    MainWindowCloseAction::Hide => {
                        if let Some(win) = app_handle.get_webview_window("main") {
                            let _ = win.hide();
                        }
                    }
                    MainWindowCloseAction::Quit => crate::app_exit::request(app_handle, 0),
                }
            }
        }
        RunEvent::ExitRequested { code, api, .. } => {
            crate::app_exit::handle_requested(app_handle, code, &api);
        }
        #[cfg(target_os = "macos")]
        RunEvent::Reopen { .. } => {
            if let Some(win) = app_handle.get_webview_window("main") {
                let _ = win.show();
                let _ = win.set_focus();
            }
        }
        _ => {}
    }
}

#[cfg(feature = "e2e")]
fn report_run_event(event: &RunEvent) {
    if let Some(name) = run_event_diagnostic(event) {
        eprintln!("[e2e-run-event] {name}");
    }
}

#[cfg(not(feature = "e2e"))]
fn report_run_event(_event: &RunEvent) {}

#[cfg(feature = "e2e")]
fn run_event_diagnostic(event: &RunEvent) -> Option<&'static str> {
    match event {
        RunEvent::Ready => Some("ready"),
        RunEvent::ExitRequested { code, .. } => Some(exit_request_diagnostic(*code)),
        RunEvent::Exit => Some("exit"),
        RunEvent::WindowEvent {
            label,
            event: WindowEvent::CloseRequested { .. },
            ..
        } if label == "main" => Some("window-close-main"),
        _ => None,
    }
}

#[cfg(feature = "e2e")]
const fn exit_request_diagnostic(code: Option<i32>) -> &'static str {
    if code.is_some() {
        "exit-requested-programmatic"
    } else {
        "exit-requested-user"
    }
}

pub fn should_start_hidden(config: &ClgoConfig) -> bool {
    should_start_hidden_for_args(config, std::env::args_os())
}

fn args_contain_autostart_marker<I, S>(args: I) -> bool
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let marker = OsStr::new(AUTOSTART_ARG);
    args.into_iter().any(|arg| arg.as_ref() == marker)
}

fn should_start_hidden_for_args<I, S>(config: &ClgoConfig, args: I) -> bool
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    config.advanced.autostart && config.advanced.start_hidden && args_contain_autostart_marker(args)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_autostart_marker() {
        assert!(args_contain_autostart_marker([
            OsStr::new("cl-go"),
            OsStr::new(AUTOSTART_ARG),
        ]));
        assert!(!args_contain_autostart_marker([OsStr::new("cl-go")]));
    }

    #[test]
    fn start_hidden_requires_autostart_setting_and_launch_marker() {
        let mut config = ClgoConfig::default();
        config.advanced.autostart = true;
        config.advanced.start_hidden = true;

        assert!(should_start_hidden_for_args(
            &config,
            [OsStr::new("cl-go"), OsStr::new(AUTOSTART_ARG)]
        ));
        assert!(!should_start_hidden_for_args(
            &config,
            [OsStr::new("cl-go")]
        ));

        config.advanced.autostart = false;
        assert!(!should_start_hidden_for_args(
            &config,
            [OsStr::new(AUTOSTART_ARG)]
        ));
    }

    #[test]
    fn main_window_close_keeps_native_macos_behavior_only() {
        assert_eq!(main_window_close_action(true), MainWindowCloseAction::Hide);
        assert_eq!(main_window_close_action(false), MainWindowCloseAction::Quit);
    }

    #[cfg(feature = "e2e")]
    #[test]
    fn e2e_run_event_diagnostics_are_fixed_categories() {
        assert_eq!(run_event_diagnostic(&RunEvent::Ready), Some("ready"));
        assert_eq!(run_event_diagnostic(&RunEvent::Exit), Some("exit"));
        assert_eq!(run_event_diagnostic(&RunEvent::Resumed), None);
        assert_eq!(
            exit_request_diagnostic(Some(0)),
            "exit-requested-programmatic"
        );
        assert_eq!(exit_request_diagnostic(None), "exit-requested-user");
    }
}
