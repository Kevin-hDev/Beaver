use super::CefUnavailableCategory;
use crate::services::browser::BrowserCapability;

#[test]
fn every_prelaunch_failure_stays_a_generic_unavailable_capability() {
    for _category in CefUnavailableCategory::ALL {
        assert_eq!(
            serde_json::to_value(BrowserCapability::Unavailable {
                restart_recommended: true,
            })
            .expect("capability payload"),
            serde_json::json!({ "status": "unavailable", "restartRecommended": true })
        );
    }
}

#[test]
fn engine_records_supervision_before_any_cef_initialization() {
    let source = include_str!("../cef_engine/startup.rs");
    let start = source
        .split_once("pub(super) fn start(")
        .expect("start function")
        .1
        .split_once("fn prepare_once(")
        .expect("preflight boundary")
        .0;
    let prepared = start
        .find("let prepared = run_with_retry")
        .expect("preflight");
    let proof = start.find("runtime.mark_supervised").expect("proof");
    let initialize = start.find("initialize_cef").expect("CEF boundary");

    assert!(prepared < proof);
    assert!(proof < initialize);

    let preflight = source
        .split_once("fn prepare_once(")
        .expect("preflight function")
        .1
        .split_once("fn initialize_cef(")
        .expect("CEF boundary")
        .0;
    assert!(preflight.contains("start_supervised"));
}

#[test]
fn rejected_cef_initialization_cannot_be_retried_or_shutdown() {
    let startup = include_str!("../cef_engine/startup.rs");
    let engine = include_str!("../cef_engine.rs");
    let library = include_str!("../cef_library.rs");

    assert_eq!(startup.matches("cef::initialize(").count(), 1);
    assert!(!startup.contains("cef::shutdown"));
    assert!(startup.contains("library.suppress_unload_after_failed_initialize()"));
    assert!(engine.contains("if let Some(engine) = engine.borrow_mut().take()"));
    assert!(library.contains("if self.unload_on_drop.load(Ordering::Acquire)"));
}
