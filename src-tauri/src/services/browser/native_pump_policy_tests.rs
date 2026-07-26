#[test]
fn native_pump_uses_a_main_thread_owner_instead_of_manual_send_sync() {
    let source = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src/services/browser/native_pump_wake.rs"),
    )
    .expect("native pump source");

    assert!(source.contains("MainThreadBound"));
    assert!(!source.contains("unsafe impl Send for PumpWake"));
    assert!(!source.contains("unsafe impl Sync for PumpWake"));
}

#[test]
fn cef_work_runs_without_holding_the_native_pump_refcell() {
    let source = include_str!("native_pump.rs");
    let work = source
        .split("fn run_message_loop_work")
        .nth(1)
        .expect("isolated CEF work function")
        .split("\nfn ")
        .next()
        .expect("CEF work function body");

    assert!(work.contains("cef::do_message_loop_work();"));
    assert!(!work.contains("borrow_mut"));
}
