use super::startup::{
    emit_vault_init_failed, prepare_macos_browser, prepare_macos_startup,
    run_before_browser_shutdown, shutdown_before_library_unload,
};
use std::cell::{Cell, RefCell};

#[test]
fn vault_init_failure_emits_only_a_signal() {
    let captured = RefCell::new(None);

    emit_vault_init_failed(|event, payload| {
        captured.replace(Some((event, payload)));
        Ok::<(), ()>(())
    });

    assert_eq!(captured.into_inner(), Some(("vault-init-failed", ())));
}

struct TestGuard<'a> {
    events: &'a RefCell<Vec<&'static str>>,
}

impl<'a> TestGuard<'a> {
    fn new(events: &'a RefCell<Vec<&'static str>>) -> Self {
        Self { events }
    }
}

impl Drop for TestGuard<'_> {
    fn drop(&mut self) {
        self.events.borrow_mut().push("unload");
    }
}

#[test]
fn macos_browser_library_loads_before_native_application() {
    let events = RefCell::new(Vec::new());

    let guard = prepare_macos_browser(
        || {
            events.borrow_mut().push("load");
            Ok(TestGuard::new(&events))
        },
        || {
            events.borrow_mut().push("prepare");
            true
        },
    );

    assert!(guard.is_some());
    assert_eq!(*events.borrow(), ["load", "prepare"]);
}

#[test]
fn macos_cef_setup_finishes_before_shell_environment_capture() {
    let events = RefCell::new(Vec::new());

    let (guard, shell_ready) = prepare_macos_startup(
        || {
            events.borrow_mut().push("load");
            Ok(TestGuard::new(&events))
        },
        || {
            events.borrow_mut().push("prepare");
            true
        },
        || {
            events.borrow_mut().push("shell");
            true
        },
    );

    assert!(guard.is_some());
    assert!(shell_ready);
    assert_eq!(*events.borrow(), ["load", "prepare", "shell"]);
}

#[test]
fn failed_library_load_skips_native_application_preparation() {
    let prepare_called = Cell::new(false);

    let guard = prepare_macos_browser(
        || Err::<(), ()>(()),
        || {
            prepare_called.set(true);
            true
        },
    );

    assert!(guard.is_none());
    assert!(!prepare_called.get());
}

#[test]
fn failed_native_application_preparation_unloads_library() {
    let events = RefCell::new(Vec::new());

    let guard = prepare_macos_browser(
        || {
            events.borrow_mut().push("load");
            Ok(TestGuard::new(&events))
        },
        || {
            events.borrow_mut().push("prepare");
            false
        },
    );

    assert!(guard.is_none());
    assert_eq!(*events.borrow(), ["load", "prepare", "unload"]);
}

#[test]
fn event_loop_precedes_browser_shutdown_and_library_unload() {
    let events = RefCell::new(Vec::new());
    let guard = TestGuard::new(&events);

    let exit_code = run_before_browser_shutdown(
        || {
            events.borrow_mut().push("event_loop");
            7
        },
        || {
            shutdown_before_library_unload(Some(guard), || {
                events.borrow_mut().push("shutdown");
            });
        },
        || events.borrow_mut().push("sweep"),
    );

    assert_eq!(exit_code, 7);
    assert_eq!(
        *events.borrow(),
        ["event_loop", "shutdown", "unload", "sweep"]
    );
}

#[test]
fn production_lifecycle_uses_the_ordered_browser_cleanup() {
    let source = include_str!("app_lifecycle.rs");
    let event_loop = source
        .find("run_before_browser_shutdown(")
        .expect("ordered event loop");
    let browser_cleanup = source
        .find("shutdown_before_library_unload(")
        .expect("ordered browser cleanup");

    assert!(event_loop < browser_cleanup);
}

#[test]
fn asynchronous_exit_cleanup_never_stops_the_browser() {
    let source = include_str!("app_exit.rs");

    assert!(!source.contains("services::browser::shutdown"));
    assert!(!source.contains("cef::shutdown"));
    assert!(source.contains("services::browser::begin_cef_shutdown"));
}

#[test]
fn ultimate_exit_is_initialized_before_tauri_builder_side_effects() {
    let source = include_str!("lib.rs");
    let coordinator = source
        .find("AppExitCoordinator::initialize()")
        .expect("ultimate coordinator initialization");
    let builder = source
        .find("tauri::Builder::default()")
        .expect("tauri builder");

    assert!(coordinator < builder);
}

#[test]
fn tray_restore_unminimizes_before_focusing() {
    let source = include_str!("tray.rs");
    let restore = source
        .find("fn restore_main_window")
        .expect("restore helper");
    let unminimize = source[restore..]
        .find("unminimize()")
        .expect("unminimize window");
    let focus = source[restore..].find("set_focus()").expect("focus window");

    assert!(unminimize < focus);
}

#[test]
fn macos_entry_point_captures_the_shell_through_cef_startup() {
    let source = include_str!("main.rs").replace("\r\n", "\n");

    assert!(source.contains(
        "let (browser_library, shell_environment_ready) = cl_go_dash_lib::prepare_macos_application();"
    ));
    assert!(source.contains(
        "#[cfg(not(target_os = \"macos\"))]\n    let shell_environment_ready = cl_go_dash_lib::initialize_shell_environment();"
    ));
}

#[test]
fn macos_engine_reuses_the_prevalidated_runtime_files() {
    let source = include_str!("services/browser/cef_engine.rs");

    assert!(source.contains("library.runtime_files()"));
    assert!(!source.contains("load_library("));
}
