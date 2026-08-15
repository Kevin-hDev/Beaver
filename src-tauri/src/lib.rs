#![cfg_attr(all(test, windows), windows_subsystem = "windows")]
// Plusieurs modules de tests compagnons portent le même nom que leur module
// parent (convention *_tests.rs). C'est intentionnel et documenté.
#![allow(clippy::module_inception)]

mod app_events;
mod app_exit;
mod app_lifecycle;
mod commands;
mod invoke_handler;
mod invoke_handler_tail;
mod models;
mod ollama_polling;
mod runtime_startup;
mod runtime_state;
mod services;
mod startup;
#[cfg(test)]
mod startup_tests;
mod storage_default_skills;
mod storage_migration;
mod storage_migration_files;
mod tray;
pub mod updater_worker;
#[cfg(all(target_os = "windows", not(feature = "windows-tests")))]
mod windows_entry;
#[cfg(all(test, target_os = "windows", feature = "windows-tests"))]
#[path = "windows_entry_plan.rs"]
mod windows_entry_plan;

use services::agent_local::ollama_client::OllamaClient;
use services::e2e_profile::{report_lifecycle, LifecycleStage};
use services::gateway::GatewayService;
use tauri::{Emitter, Manager};

pub use runtime_state::ActiveStreams;
#[cfg(target_os = "macos")]
pub use services::browser::BrowserLibraryGuard;
#[cfg(target_os = "macos")]
pub fn run_macos_cef_helper() -> std::process::ExitCode {
    services::browser::run_macos_cef_helper()
}
#[cfg(all(target_os = "windows", not(feature = "windows-tests")))]
pub use startup::launch_windows_browser_host;
#[cfg(target_os = "macos")]
pub use startup::prepare_macos_application;
pub use startup::{
    configure_git_network_policy, initialize_shell_environment, prepare_browser_native_application,
    run, run_shell_sandbox_helper,
};

pub(crate) fn run_inner(
    #[cfg(target_os = "macos")] browser_library: Option<BrowserLibraryGuard>,
) -> bool {
    let exit_coordinator = match app_exit::AppExitCoordinator::initialize() {
        Ok(coordinator) => coordinator,
        Err(_) => {
            eprintln!("[exit] safety initialization unavailable");
            return false;
        }
    };
    if services::mcp_bridge::process_manager::init(exit_coordinator.work_supervisor()).is_err() {
        eprintln!("[mcp] shutdown supervision unavailable");
        return false;
    }
    let runtime = runtime_state::services(&exit_coordinator);
    std::hint::black_box(tauri::utils::platform::bundle_type());
    let builder = tauri::Builder::default()
        .plugin(services::app_log::plugin())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(services::autostart_migration::plugin())
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            runtime_state::show_main_window(app);
        }));
    #[cfg(feature = "e2e")]
    let builder = builder.plugin(tauri_plugin_wdio::init());
    #[cfg(feature = "e2e")]
    let builder = builder.plugin(tauri_plugin_wdio_webdriver::init());
    let ollama_manager = runtime.ollama.clone();
    let app = builder
        .manage(OllamaClient::new(ollama_manager))
        .manage(runtime.ollama)
        .manage(exit_coordinator)
        .manage(runtime.agent_work)
        .manage(runtime.oauth_work)
        .manage(ActiveStreams(Default::default()))
        .manage(services::mascot::MascotRuntime::default())
        .manage(runtime.downloads)
        .manage(runtime.app_update)
        .manage(runtime.searxng)
        .manage(runtime.terminal)
        .manage(runtime.background)
        .manage(services::browser::BrowserRuntimeHandle::default())
        .manage(services::browser::BrowserSessionService::default())
        .manage(services::browser::LocalSiteScanner::default())
        .manage(runtime.gateway)
        .manage(commands::file_tree_watcher::FileTreeWatcher::new())
        .manage(runtime.forecast)
        .on_page_load(|webview, payload| {
            if webview.label() == "main"
                && matches!(payload.event(), tauri::webview::PageLoadEvent::Started)
            {
                services::browser::reset_page_surface(webview.app_handle());
            }
        })
        .setup(|app| {
            report_lifecycle(LifecycleStage::SetupEntered);
            let startup_cutoff = chrono::Utc::now();
            let background = app
                .state::<services::runtime_background::RuntimeBackgroundServices>()
                .inner()
                .clone();
            runtime_state::initialize_agent_runtime(app.handle())?;
            storage_migration::initialize(app.handle()).map_err(std::io::Error::other)?;
            report_lifecycle(LifecycleStage::StorageInitialized);
            if services::agent_local::directory_access::initialize_policy().is_err() {
                ::log::error!("[directory-access] policy unavailable");
            }
            runtime_startup::start_recovery(&background, startup_cutoff);
            if services::security_cleanup::run().is_err() {
                ::log::error!("[security cleanup] cleanup failed");
            }
            report_lifecycle(LifecycleStage::RecoveryStarted);
            services::e2e_profile::load_dotenv(|| {
                let _ = dotenvy::dotenv();
            });
            if services::api_keys::init_for_runtime().is_err() {
                ::log::error!("[vault] init failed");
                let handle = app.handle().clone();
                let _ = background.spawn_task(move |cancel| async move {
                    tokio::select! {
                        _ = cancel.cancelled() => {}
                        _ = tokio::time::sleep(std::time::Duration::from_secs(2)) => {
                            startup::emit_vault_init_failed(|event, payload| handle.emit(event, payload));
                        }
                    }
                });
            }
            report_lifecycle(LifecycleStage::VaultInitialized);
            services::e2e_profile::run_host_mutation(|| {
                services::extensions::initialize_on_startup(app.handle());
                services::searxng::prepare_on_startup(app.handle().clone());
                runtime_startup::start_ollama(&background, app.handle());
            });

            let config = services::config::read_config().unwrap_or_default();
            report_lifecycle(LifecycleStage::ConfigLoaded);
            services::mascot::initialize(app.handle(), config.mascot.clone());
            services::mascot::start_activity_cleanup(app.handle());
            report_lifecycle(LifecycleStage::MascotStarted);

            services::e2e_profile::run_host_mutation(|| {
                services::autostart_migration::synchronize_at_startup(
                    app.handle(),
                    config.advanced.autostart,
                );
            });

            // Start hidden applies only to launches initiated by the autostart entry.
            if app_events::should_start_hidden(&config) {
                if let Some(win) = app.get_webview_window("main") {
                    let _ = win.hide();
                }
            }

            // Linux/Windows : désactiver les décorations natives, boutons custom React
            #[cfg(not(target_os = "macos"))]
            {
                if let Some(win) = app.get_webview_window("main") {
                    let _ = win.set_decorations(false);
                    let _ = win.set_shadow(false);
                }
            }

            // Tray icon
            if config.advanced.show_tray {
                let _ = tray::create_tray(app);
            }

            // Gateway : démarrage si configuré
            if config.gateway.enabled && config.gateway.start_with_app {
                let gw_config = config.gateway.clone();
                let gw_handle = app.handle().clone();
                let _ = background.spawn_task(move |cancel| async move {
                    tokio::select! {
                        _ = cancel.cancelled() => {}
                        _ = async {
                            let gw = gw_handle.state::<GatewayService>();
                            let _ = gw.start(gw_config, gw_handle.clone()).await;
                        } => {}
                    }
                });
            }

            report_lifecycle(LifecycleStage::WindowConfigured);
            services::file_watcher::start(app.handle());
            report_lifecycle(LifecycleStage::FileWatcherStarted);
            let scheduler = runtime_state::scheduler(app.handle())?;
            app.manage(scheduler);
            report_lifecycle(LifecycleStage::SchedulerStarted);
            services::e2e_profile::run_host_mutation(|| {
                ollama_polling::start(app.handle().clone());
                runtime_startup::start_litellm(&background);
            });
            services::update_health::acknowledge_from_args(std::env::args_os())
                .map_err(std::io::Error::other)?;
            report_lifecycle(LifecycleStage::SetupCompleted);
            Ok(())
        })
        .on_window_event(|_window, _event| {
            if let tauri::WindowEvent::Focused(focused) = _event {
                services::mascot::handle_window_focus(_window.app_handle(), *focused);
            }
        })
        .invoke_handler(invoke_handler::for_build!())
        .build(tauri::generate_context!())
        .expect("error while building tauri application");

    let exit_code = app_lifecycle::run(
        app,
        #[cfg(target_os = "macos")]
        browser_library,
    );
    std::process::exit(exit_code);
}
