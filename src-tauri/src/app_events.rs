use crate::services::gateway::GatewayService;
use crate::services::{ollama_kill, ollama_lifecycle};
use crate::{models::ClgoConfig, services};
use std::ffi::OsStr;
use tauri::{Manager, RunEvent, WindowEvent};

pub const AUTOSTART_ARG: &str = "--clgo-autostart";

pub fn handle_run_event(app_handle: &tauri::AppHandle, event: RunEvent) {
    match event {
        RunEvent::WindowEvent {
            label,
            event: WindowEvent::CloseRequested { .. },
            ..
        } => {
            if label == "main" {
                #[cfg(not(target_os = "macos"))]
                {
                    if should_hide_instead_of_quit() {
                        if let Some(win) = app_handle.get_webview_window("main") {
                            let _ = win.hide();
                        }
                    } else {
                        cleanup(app_handle);
                    }
                }
            }
        }
        RunEvent::ExitRequested { .. } | RunEvent::Exit => {
            cleanup(app_handle);
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

fn cleanup(app_handle: &tauri::AppHandle) {
    tauri::async_runtime::block_on(services::oauth_providers::cancel_all());
    if let Some(gw) = app_handle.try_state::<GatewayService>() {
        let gw = gw.inner();
        tauri::async_runtime::block_on(async { gw.stop().await });
    }
    services::mcp_bridge::process_manager::shutdown_all();
    if let Some(pty) = app_handle.try_state::<services::terminal::PtyManager>() {
        pty.kill_all();
    }
    if let Some(chronos) = app_handle.try_state::<services::forecast::sidecar::ChronosSidecar>() {
        tauri::async_runtime::block_on(async {
            services::forecast::sidecar::stop(chronos.inner()).await;
        });
    }
    if let Some(searxng) = app_handle.try_state::<services::searxng::SearxngSidecar>() {
        tauri::async_runtime::block_on(async {
            services::searxng::stop(searxng.inner()).await;
        });
    }
    ollama_kill::release_vram_blocking();
    ollama_lifecycle::stop_sidecar(app_handle);
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
