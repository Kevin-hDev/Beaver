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
use services::gateway::GatewayService;
use services::ollama_lifecycle::{self, OllamaSidecar};
use services::scheduler::Scheduler;
use tauri::{Emitter, Manager};

pub use runtime_state::ActiveStreams;
#[cfg(target_os = "macos")]
pub use services::browser::BrowserLibraryGuard;
#[cfg(all(target_os = "windows", not(feature = "windows-tests")))]
pub use startup::launch_windows_browser_host;
#[cfg(target_os = "macos")]
pub use startup::prepare_macos_application;
pub use startup::{
    configure_git_network_policy, initialize_shell_environment, prepare_browser_native_application,
    run, run_shell_sandbox_helper,
};

static STREAM_GENERATION: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

pub(crate) fn run_inner(#[cfg(target_os = "macos")] browser_library: Option<BrowserLibraryGuard>) {
    std::hint::black_box(tauri::utils::platform::bundle_type());
    let builder = tauri::Builder::default()
        .plugin(services::app_log::plugin())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(services::autostart_migration::plugin())
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(w) = app.get_webview_window("main") {
                let _ = w.show();
                let _ = w.unminimize();
                let _ = w.set_focus();
            }
        }));
    #[cfg(feature = "e2e")]
    let builder = builder.plugin(tauri_plugin_wdio::init());
    #[cfg(feature = "e2e")]
    let builder = builder.plugin(tauri_plugin_wdio_webdriver::init());
    let app = builder
        .manage(OllamaClient::new())
        .manage(app_exit::AppExitCoordinator::default())
        .manage(ActiveStreams(Default::default()))
        .manage(services::mascot::MascotRuntime::default())
        .manage(OllamaSidecar::new())
        .manage(services::model_downloads::ModelDownloadManager::new())
        .manage(services::searxng::SearxngSidecar::new())
        .manage(services::terminal::PtyManager::new())
        .manage(services::browser::BrowserRuntimeHandle::default())
        .manage(services::browser::BrowserSessionService::default())
        .manage(services::browser::LocalSiteScanner::default())
        .manage(GatewayService::new())
        .manage(commands::file_tree_watcher::FileTreeWatcher::new())
        .manage(services::forecast::sidecar::ChronosSidecar::new())
        .on_page_load(|webview, payload| {
            if webview.label() == "main"
                && matches!(payload.event(), tauri::webview::PageLoadEvent::Started)
            {
                services::browser::reset_page_surface(webview.app_handle());
            }
        })
        .setup(|app| {
            let startup_cutoff = chrono::Utc::now();
            services::agent_local::shell_sandbox::cleanup_stale();
            services::agent_local::app_handle_global::init(app.handle().clone());
            services::agent_local::subagent_spawn_channel::init();
            storage_migration::initialize(app.handle()).map_err(std::io::Error::other)?;
            if services::agent_local::directory_access::initialize_policy().is_err() {
                ::log::error!("[directory-access] policy unavailable");
            }
            tauri::async_runtime::spawn(async {
                if services::forecast::notes_cleanup::recover_pending_deletions()
                    .await
                    .is_err()
                {
                    ::log::warn!("[forecast] récupération des notes différée");
                }
            });
            if services::security_cleanup::run().is_err() {
                ::log::warn!("[security cleanup] cleanup failed");
            }
            // Cleanup des sous-agents orphelins (crash précédent) : non bloquant.
            tauri::async_runtime::spawn(async move {
                services::agent_local::subagent_startup_cleanup::cleanup_orphans(startup_cutoff)
                    .await;
            });
            services::e2e_profile::load_dotenv(|| {
                let _ = dotenvy::dotenv();
            });
            if services::api_keys::init_for_runtime().is_err() {
                ::log::error!("[vault] init failed");
                let handle = app.handle().clone();
                tauri::async_runtime::spawn(async move {
                    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                    startup::emit_vault_init_failed(|event, payload| handle.emit(event, payload));
                });
            }
            services::e2e_profile::run_host_mutation(|| {
                services::extensions::initialize_on_startup(app.handle());
                services::searxng::prepare_on_startup(app.handle().clone());
                if ollama_lifecycle::ollama_binary_path().is_ok() {
                    let handle = app.handle().clone();
                    tauri::async_runtime::spawn(async move {
                        match tokio::task::spawn_blocking(move || {
                            ollama_lifecycle::start_sidecar(&handle)
                        })
                        .await
                        {
                            Ok(Err(e)) => ::log::warn!("[ollama] sidecar start failed: {}", e),
                            Err(e) => ::log::warn!("[ollama] sidecar task failed: {}", e),
                            _ => {}
                        }
                    });
                }
            });

            let config = services::config::read_config().unwrap_or_default();
            services::mascot::initialize(app.handle(), config.mascot.clone());
            services::mascot::start_activity_cleanup(app.handle().clone());

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
                tauri::async_runtime::spawn(async move {
                    let gw = gw_handle.state::<GatewayService>();
                    let _ = gw.start(gw_config, gw_handle.clone()).await;
                });
            }

            services::file_watcher::start(app.handle());
            let scheduler = Scheduler::spawn(app.handle().clone());
            app.manage(scheduler);
            services::e2e_profile::run_host_mutation(|| {
                ollama_polling::start(app.handle().clone());
                tauri::async_runtime::spawn(services::llm::litellm_catalog::init());
            });
            services::update_health::acknowledge_from_args(std::env::args_os())
                .map_err(std::io::Error::other)?;
            Ok(())
        })
        .on_window_event(|_window, _event| {
            if let tauri::WindowEvent::Focused(focused) = _event {
                services::mascot::handle_window_focus(_window.app_handle(), *focused);
            }
            #[cfg(target_os = "macos")]
            if let tauri::WindowEvent::CloseRequested { api, .. } = _event {
                if _window.label() == "main" {
                    let _ = _window.hide();
                    api.prevent_close();
                }
            }
        })
        .invoke_handler(invoke_handler::generate!())
        .build(tauri::generate_context!())
        .expect("error while building tauri application");

    let exit_code = app_lifecycle::run(
        app,
        #[cfg(target_os = "macos")]
        browser_library,
    );
    std::process::exit(exit_code);
}
