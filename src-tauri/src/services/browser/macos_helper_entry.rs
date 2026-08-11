use super::cef_supervision::{parse_helper_marker, CefUnavailableCategory, MacHelperBootstrap};
use cef::{args::Args, *};
use zeroize::Zeroize;

pub(crate) fn run() -> std::process::ExitCode {
    let mut marker = match parse_helper_marker() {
        Ok(marker) => marker,
        Err(_) => return setup_failed(CefUnavailableCategory::Admission),
    };
    let bootstrap = match MacHelperBootstrap::prepare(
        &marker,
        &super::cef_runtime_policy::cef_supervision_root(),
    ) {
        Ok(bootstrap) => bootstrap,
        Err(category) => return setup_failed(category),
    };
    marker.zeroize();
    let args = Args::new();
    let mut sandbox = cef::sandbox::Sandbox::new();
    sandbox.initialize(args.as_main_args());
    let admission = match bootstrap.admit_after_sandbox() {
        Ok(admission) => admission,
        Err(category) => return setup_failed(category),
    };
    let executable = match std::env::current_exe() {
        Ok(executable) => executable,
        Err(_) => return setup_failed(CefUnavailableCategory::Object),
    };
    let loader = cef::library_loader::LibraryLoader::new(&executable, true);
    if !loader.load() {
        return setup_failed(CefUnavailableCategory::Sandbox);
    }
    let _ = api_hash(sys::CEF_API_VERSION_LAST, 0);
    let mut app = AdmittedMacHelperApp::new();
    let code = execute_process(
        Some(args.as_main_args()),
        Some(&mut app),
        std::ptr::null_mut(),
    );
    drop(admission);
    if !(0..=u8::MAX.into()).contains(&code) {
        std::process::ExitCode::FAILURE
    } else {
        std::process::ExitCode::from(code as u8)
    }
}

fn setup_failed(category: CefUnavailableCategory) -> std::process::ExitCode {
    eprintln!("[browser-helper] setup failed ({})", category.code());
    std::process::ExitCode::FAILURE
}

wrap_app! {
    struct AdmittedMacHelperApp {}

    impl App {
        fn on_before_command_line_processing(
            &self,
            _process_type: Option<&CefString>,
            command_line: Option<&mut CommandLine>,
        ) {
            if let Some(command_line) = command_line {
                command_line.remove_switch(Some(&CefString::from(
                    super::cef_supervision::CEF_ADMISSION_SWITCH,
                )));
            }
        }
    }
}
