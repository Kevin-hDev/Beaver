use tauri::{Manager, Runtime};

pub mod commands;
pub mod contract;
pub mod download;
pub mod error;
pub mod install;
mod install_platform;
mod install_trace;

fn configure<R: Runtime>(builder: tauri::Builder<R>) -> tauri::Builder<R> {
    let locale = normalized_locale(sys_locale::get_locale().as_deref());
    let script = format!("window.__BEAVER_INSTALLER_LOCALE__={locale:?};");
    builder
        .append_invoke_initialization_script(script)
        .plugin(tauri_plugin_dialog::init())
}

fn normalized_locale(locale: Option<&str>) -> &'static str {
    match locale
        .unwrap_or_default()
        .split(['-', '_'])
        .next()
        .unwrap_or_default()
        .to_ascii_lowercase()
        .as_str()
    {
        "de" => "de",
        "es" => "es",
        "fr" => "fr",
        "it" => "it",
        "ja" => "ja",
        "zh" => "zh",
        _ => "en",
    }
}

pub mod launch_args;
pub mod platform;
pub mod process;
pub mod runtime;
pub mod temp_ownership;
pub mod trace;

pub fn run() {
    let launch = match launch_args::LaunchContext::parse_from(std::env::args_os()) {
        Ok(launch) => launch,
        Err(error) => exit_with(error),
    };
    let temp_root = std::env::temp_dir();
    temp_ownership::purge_orphans(&temp_root, &launch.run_id);
    let run =
        match temp_ownership::OwnedTempRun::adopt(&temp_root, &launch.work_dir, &launch.run_id) {
            Ok(run) => run,
            Err(error) => exit_with(error),
        };
    let service = match install::InstallerService::new(launch, run) {
        Ok(service) => service,
        Err(error) => exit_with(error),
    };
    let app = configure(tauri::Builder::default().manage(service))
        .invoke_handler(tauri::generate_handler![
            commands::installer_snapshot,
            commands::choose_install_directory,
            commands::start_install,
            commands::cancel_install,
            commands::launch_beaver,
        ])
        .build(tauri::generate_context!());
    let app = match app {
        Ok(app) => app,
        Err(_) => {
            eprintln!("installer-runtime-failed");
            std::process::exit(1);
        }
    };
    let exit_code = app.run_return(|app, event| {
        let service = app.state::<install::InstallerService>();
        match event {
            tauri::RunEvent::WindowEvent {
                label,
                event: tauri::WindowEvent::CloseRequested { api, .. },
                ..
            } if label == "main" && service.operation_active() => api.prevent_close(),
            tauri::RunEvent::ExitRequested { api, .. }
                if service.operation_active() || service.shutdown().is_err() =>
            {
                api.prevent_exit();
            }
            _ => {}
        }
    });
    std::process::exit(exit_code);
}

fn exit_with(error: error::InstallerError) -> ! {
    eprintln!("{}", error.code());
    std::process::exit(1)
}

#[cfg(test)]
#[path = "launch_args_tests.rs"]
mod launch_args_tests;

#[cfg(test)]
#[path = "temp_ownership_tests.rs"]
mod temp_ownership_tests;

#[cfg(test)]
#[path = "download_tests.rs"]
mod download_tests;

#[cfg(test)]
#[path = "trace_tests.rs"]
mod trace_tests;

#[cfg(test)]
#[path = "contract_tests.rs"]
mod contract_tests;

#[cfg(test)]
#[path = "runtime_tests.rs"]
mod runtime_tests;

#[cfg(test)]
#[path = "process_tests.rs"]
mod process_tests;

#[cfg(test)]
#[path = "platform_tests.rs"]
mod platform_tests;

#[cfg(all(test, target_os = "macos"))]
#[path = "platform_macos_tests.rs"]
mod platform_macos_tests;

#[cfg(test)]
mod tests {
    #[test]
    fn installer_locale_is_bounded_and_normalized() {
        assert_eq!(super::normalized_locale(Some("fr-FR")), "fr");
        assert_eq!(super::normalized_locale(Some("pt_BR")), "en");
    }

    #[test]
    fn minimal_application_builds() {
        super::configure(tauri::test::mock_builder())
            .build(tauri::test::mock_context(tauri::test::noop_assets()))
            .expect("build minimal installer");
    }
}
