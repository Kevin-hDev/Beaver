use crate::app_build_mode::AppBuildMode;
use crate::runtime_state::{ActiveStreams, RuntimeServices};
use crate::services::agent_local::ollama_client::OllamaClient;
use tauri::Manager;

pub(super) fn build(
    exit_coordinator: crate::app_exit::AppExitCoordinator,
    runtime: RuntimeServices,
    ui_startup: crate::services::extensions::UiStartupState,
) -> tauri::Result<tauri::App<tauri::Wry>> {
    build_with_mode(
        exit_coordinator,
        runtime,
        ui_startup,
        AppBuildMode::Interactive,
    )
}

#[cfg(debug_assertions)]
pub(super) fn build_live_fixture(
    exit_coordinator: crate::app_exit::AppExitCoordinator,
    runtime: RuntimeServices,
) -> tauri::Result<tauri::App<tauri::Wry>> {
    build_with_mode(
        exit_coordinator,
        runtime,
        crate::services::extensions::UiStartupState::resolved(
            crate::services::extensions::UiStartupMode::Normal,
        ),
        AppBuildMode::LiveFixture,
    )
}

fn build_with_mode(
    exit_coordinator: crate::app_exit::AppExitCoordinator,
    runtime: RuntimeServices,
    ui_startup: crate::services::extensions::UiStartupState,
    mode: AppBuildMode,
) -> tauri::Result<tauri::App<tauri::Wry>> {
    let builder = tauri::Builder::default()
        .plugin(crate::services::app_log::plugin())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init());
    // Live validation must coexist with the already-open development app.
    let builder = if mode.installs_single_instance() {
        builder.plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            crate::runtime_state::show_main_window(app);
        }))
    } else {
        builder
    };
    #[cfg(target_os = "macos")]
    let builder = builder
        .menu(crate::macos_app_menu::build)
        .on_menu_event(crate::macos_app_menu::handle_event);
    #[cfg(feature = "e2e")]
    let builder = builder.plugin(tauri_plugin_wdio::init());
    #[cfg(feature = "e2e")]
    let builder = builder.plugin(tauri_plugin_wdio_webdriver::init());
    let builder = crate::services::extensions::ui_protocol::register(builder, ui_startup.clone());
    let ollama_manager = runtime.ollama.clone();
    let builder = builder
        .manage(OllamaClient::new(ollama_manager))
        .manage(runtime.ollama)
        .manage(exit_coordinator)
        .manage(runtime.agent_work)
        .manage(runtime.oauth_work)
        .manage(ActiveStreams(Default::default()))
        .manage(crate::services::mascot::MascotRuntime::default())
        .manage(runtime.downloads)
        .manage(runtime.app_update)
        .manage(runtime.update_progress)
        .manage(runtime.searxng)
        .manage(runtime.terminal)
        .manage(runtime.background)
        .manage(runtime.voice)
        .manage(crate::services::browser::BrowserRuntimeHandle::default())
        .manage(crate::services::browser::BrowserSessionService::default())
        .manage(crate::services::browser::LocalSiteScanner::default())
        .manage(ui_startup);
    #[cfg(any(target_os = "macos", windows))]
    let builder =
        builder.manage(crate::services::voice::capture::window_events::WindowEventState::default());
    builder
        .manage(crate::services::extensions::UiLoadAcknowledger::new())
        .manage(runtime.gateway)
        .manage(crate::commands::file_tree_watcher::FileTreeWatcher::new())
        .manage(runtime.forecast)
        .on_page_load(|webview, payload| {
            if webview.label() == "main"
                && matches!(payload.event(), tauri::webview::PageLoadEvent::Started)
            {
                crate::services::browser::reset_page_surface(webview.app_handle());
            }
        })
        .setup(crate::app_setup::setup)
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::Focused(focused) = event {
                crate::services::mascot::handle_window_focus(window.app_handle(), *focused);
                #[cfg(any(target_os = "macos", windows))]
                crate::services::voice::capture::window_events::record_for_app(
                    window.app_handle(),
                    crate::services::voice::capture::window_events::WindowSignal::FocusChanged(
                        *focused,
                    ),
                );
            }
            #[cfg(any(target_os = "macos", windows))]
            if matches!(event, tauri::WindowEvent::Resized(_))
                && window.is_minimized().unwrap_or(false)
            {
                crate::services::voice::capture::window_events::record_for_app(
                    window.app_handle(),
                    crate::services::voice::capture::window_events::WindowSignal::Minimized,
                );
            }
        })
        .invoke_handler(crate::invoke_gate::wrap(
            crate::invoke_handler::for_build!(),
        ))
        .build(tauri::generate_context!())
}

#[cfg(test)]
#[path = "app_build_tests.rs"]
mod tests;
