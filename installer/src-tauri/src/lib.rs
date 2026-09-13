use tauri::Runtime;

pub mod download;
pub mod error;

fn configure<R: Runtime>(builder: tauri::Builder<R>) -> tauri::Builder<R> {
    builder.plugin(tauri_plugin_dialog::init())
}

pub mod launch_args;
pub mod temp_ownership;
pub mod trace;

pub fn run() {
    configure(tauri::Builder::default())
        .run(tauri::generate_context!())
        .expect("failed to run Beaver Installer");
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
mod tests {
    #[test]
    fn minimal_application_builds() {
        super::configure(tauri::test::mock_builder())
            .build(tauri::test::mock_context(tauri::test::noop_assets()))
            .expect("build minimal installer");
    }
}
