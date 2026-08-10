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
}
