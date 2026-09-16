use crate::services::e2e_profile::{report_lifecycle, LifecycleStage};
use crate::services::gateway::GatewayService;
use tauri::{Emitter, Manager};

pub(super) fn setup(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(any(target_os = "macos", windows))]
    app.state::<crate::services::voice::runtime::VoiceRuntime>()
        .attach_app(app.handle());
    #[cfg(target_os = "macos")]
    crate::macos_termination::install(app.handle())
        .map_err(|_| std::io::Error::other("native termination hook unavailable"))?;
    report_lifecycle(LifecycleStage::SetupEntered);
    let startup_cutoff = chrono::Utc::now();
    let background = app
        .state::<crate::services::runtime_background::RuntimeBackgroundServices>()
        .inner()
        .clone();
    crate::runtime_state::initialize_agent_runtime(app.handle())?;
    crate::storage_migration::initialize(app.handle()).map_err(std::io::Error::other)?;
    #[cfg(any(target_os = "macos", windows))]
    if let Some(downloads) =
        app.try_state::<crate::services::model_downloads::ModelDownloadManager>()
    {
        if crate::services::voice::download::resume_incomplete_removals(
            &crate::services::paths::data_dir(),
        )
        .is_err()
        {
            ::log::warn!("[voice] incomplete model cleanup remains pending");
        }
        let resource_dir = app.path().resource_dir()?;
        downloads.restore_voice_checkpoints(&crate::services::paths::data_dir(), &resource_dir);
    }
    report_lifecycle(LifecycleStage::StorageInitialized);
    if crate::services::agent_local::directory_access::initialize_policy().is_err() {
        ::log::error!("[directory-access] policy unavailable");
    }
    crate::runtime_startup::start_recovery(&background, startup_cutoff);
    if crate::services::security_cleanup::run().is_err() {
        ::log::error!("[security cleanup] cleanup failed");
    }
    report_lifecycle(LifecycleStage::RecoveryStarted);
    crate::services::e2e_profile::load_dotenv(|| {
        let _ = dotenvy::dotenv();
    });
    if crate::services::api_keys::init_for_runtime().is_err() {
        ::log::error!("[vault] init failed");
        let handle = app.handle().clone();
        let _ = background.spawn_task(move |cancel| async move {
            tokio::select! {
                _ = cancel.cancelled() => {}
                _ = tokio::time::sleep(std::time::Duration::from_secs(2)) => {
                    crate::startup::emit_vault_init_failed(|event, payload| handle.emit(event, payload));
                }
            }
        });
    }
    report_lifecycle(LifecycleStage::VaultInitialized);
    crate::services::e2e_profile::run_host_mutation(|| {
        crate::services::extensions::initialize_on_startup(app.handle());
        crate::services::searxng::prepare_on_startup(app.handle().clone());
        crate::runtime_startup::start_gpu_memory(&background);
        crate::runtime_startup::start_ollama(&background, app.handle());
    });
    configure_application(app, &background)?;
    report_lifecycle(LifecycleStage::SetupCompleted);
    Ok(())
}

fn configure_application(
    app: &mut tauri::App,
    background: &crate::services::runtime_background::RuntimeBackgroundServices,
) -> Result<(), Box<dyn std::error::Error>> {
    let config = crate::services::config::read_config().unwrap_or_default();
    report_lifecycle(LifecycleStage::ConfigLoaded);
    crate::services::mascot::initialize(app.handle(), config.mascot.clone());
    crate::services::mascot::start_activity_cleanup(app.handle());
    report_lifecycle(LifecycleStage::MascotStarted);
    crate::services::e2e_profile::run_host_mutation(|| {
        crate::services::autostart_migration::synchronize_at_startup(
            app.handle(),
            config.advanced.autostart,
        );
    });
    // Only the OS autostart entry may hide the main window at launch.
    if crate::app_events::should_start_hidden(&config) {
        if let Some(window) = app.get_webview_window("main") {
            let _ = window.hide();
        }
    }
    // Linux and Windows use the React title bar instead of native decorations.
    #[cfg(not(target_os = "macos"))]
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.set_decorations(false);
        let _ = window.set_shadow(false);
    }
    if config.advanced.show_tray {
        let _ = crate::tray::create_tray(app);
    }
    // The gateway starts only when both the feature and startup policy opt in.
    if config.gateway.enabled && config.gateway.start_with_app {
        let gateway_config = config.gateway.clone();
        let handle = app.handle().clone();
        let _ = background.spawn_task(move |cancel| async move {
            tokio::select! {
                _ = cancel.cancelled() => {}
                _ = async {
                    let gateway = handle.state::<GatewayService>();
                    let _ = gateway.start(gateway_config, handle.clone()).await;
                } => {}
            }
        });
    }
    report_lifecycle(LifecycleStage::WindowConfigured);
    crate::services::file_watcher::start(app.handle());
    report_lifecycle(LifecycleStage::FileWatcherStarted);
    let scheduler = crate::runtime_state::scheduler(app.handle())?;
    app.manage(scheduler);
    report_lifecycle(LifecycleStage::SchedulerStarted);
    crate::services::e2e_profile::run_host_mutation(|| {
        crate::runtime_state::start_ollama_polling(app.handle());
        crate::runtime_startup::start_litellm(background);
    });
    crate::services::update_health::acknowledge_from_args(std::env::args_os())
        .map_err(std::io::Error::other)?;
    crate::storage_migration::acknowledge_automation_migration().map_err(std::io::Error::other)?;
    Ok(())
}
