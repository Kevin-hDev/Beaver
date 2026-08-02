use crate::models::ClgoConfig;
#[cfg(not(target_os = "macos"))]
use crate::services;
use std::ffi::OsStr;
use tauri::{Manager, RunEvent, WindowEvent};

pub const AUTOSTART_ARG: &str = "--clgo-autostart";

pub fn handle_run_event(app_handle: &tauri::AppHandle, event: RunEvent) {
    match event {
        RunEvent::WindowEvent {
            label,
            event: WindowEvent::CloseRequested { api, .. },
            ..
        } => {
            if label == "main" {
                #[cfg(target_os = "macos")]
                let _ = api;
                #[cfg(not(target_os = "macos"))]
                {
                    api.prevent_close();
                    if should_hide_instead_of_quit() {
                        if let Some(win) = app_handle.get_webview_window("main") {
                            let _ = win.hide();
                        }
                    } else {
                        crate::app_exit::request(app_handle, 0);
                    }
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

#[cfg(not(target_os = "macos"))]
fn should_hide_instead_of_quit() -> bool {
    let config = services::config::read_config().unwrap_or_default();
    let gateway_active = config.gateway.enabled && config.gateway.run_when_window_closed;
    let tray_visible = config.advanced.show_tray;
    gateway_active && tray_visible
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
}
