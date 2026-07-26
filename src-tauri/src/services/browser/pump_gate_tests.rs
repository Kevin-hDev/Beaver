use super::pump_gate::PumpGate;
use super::pump_scheduler::fallback_pump_interval_ms;

#[test]
fn native_fallback_keeps_cef_responsive() {
    assert_eq!(fallback_pump_interval_ms(), 33);
}

#[test]
fn message_pump_allows_only_one_scheduled_job() {
    let gate = PumpGate::default();

    assert!(gate.request());
    assert!(!gate.request());
    assert!(gate.begin_dispatch());
    assert!(!gate.begin_dispatch());
    assert!(!gate.request());
    assert!(gate.complete_and_requeue());
    assert!(gate.begin_dispatch());
    assert!(!gate.complete_and_requeue());
}

#[test]
fn stopped_message_pump_fails_closed() {
    let gate = PumpGate::default();

    assert!(!gate.is_stopped());
    gate.stop();
    assert!(gate.is_stopped());
    assert!(!gate.request());
    assert!(!gate.begin_dispatch());
    assert!(!gate.complete_and_requeue());
    assert!(!gate.request());
}

#[test]
fn macos_pump_uses_one_shot_work_on_the_owner_run_loop() {
    let source = include_str!("native_pump.rs");
    let timer = source
        .split("timerWithTimeInterval_target_selector_userInfo_repeats")
        .nth(1)
        .expect("one-shot timer creation")
        .split(");")
        .next()
        .expect("bounded timer arguments");

    assert!(source.contains("performSelector_onThread_withObject_waitUntilDone"));
    assert!(source.contains("NSRunLoop::currentRunLoop"));
    assert!(timer.contains("None,"));
    assert!(timer.contains("false,"));
}

#[test]
fn cef_work_runs_after_the_tauri_wakeup_to_avoid_shutdown_reentrancy() {
    let pump = include_str!("native_pump.rs");

    assert!(!pump.contains("run_on_main_thread"));
    assert!(pump.contains("performSelector_onThread_withObject_waitUntilDone"));
}
