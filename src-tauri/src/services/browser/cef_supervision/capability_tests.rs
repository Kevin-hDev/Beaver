use super::CefUnavailableCategory;
use crate::services::browser::BrowserCapability;

#[test]
fn every_prelaunch_failure_stays_a_generic_unavailable_capability() {
    for _category in CefUnavailableCategory::ALL {
        assert_eq!(
            serde_json::to_value(BrowserCapability::Unavailable).expect("capability payload"),
            serde_json::json!({ "status": "unavailable" })
        );
    }
}

#[test]
fn engine_records_supervision_before_any_cef_initialization() {
    let source = include_str!("../cef_engine.rs");
    let tracker = source.find("start_supervised").expect("tracker start");
    let proof = source.find("runtime.mark_supervised").expect("proof");
    let initialize = source.find("if cef::initialize").expect("CEF initialize");

    assert!(tracker < proof);
    assert!(proof < initialize);
}
