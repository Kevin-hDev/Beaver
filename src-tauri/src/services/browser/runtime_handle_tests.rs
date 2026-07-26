use super::runtime_handle::{BrowserCapability, BrowserRuntimeHandle, CEF_VERSION};

#[test]
fn browser_initialization_waits_for_the_ready_event() {
    let source = include_str!("../../lib.rs");

    assert!(!source.contains("services::browser::setup(app);"));
    assert!(source.contains("services::browser::setup_on_run_event(app_handle, &event);"));
}

#[test]
fn capability_events_apply_the_same_security_gate_as_commands() {
    let source = include_str!("cef_cookie_gate.rs");

    assert!(source.contains("super::capability_for_runtime(&self.runtime)"));
    assert!(!source.contains("self.runtime.capability()"));
}

#[test]
fn runtime_becomes_ready_only_after_ordered_bootstrap() {
    let runtime = BrowserRuntimeHandle::default();

    assert_eq!(runtime.capability(), BrowserCapability::Unavailable);
    assert!(runtime.mark_application_prepared());
    assert!(runtime.mark_running());
    assert_eq!(
        runtime.capability(),
        BrowserCapability::Ready {
            engine_version: CEF_VERSION.to_string(),
        }
    );
}

#[test]
fn invalid_runtime_transition_fails_closed_for_all_clones() {
    let runtime = BrowserRuntimeHandle::default();
    let clone = runtime.clone();

    assert!(!runtime.mark_running());
    assert_eq!(clone.capability(), BrowserCapability::Unavailable);
}

#[test]
fn native_surface_is_allowed_only_after_the_security_gate() {
    let runtime = BrowserRuntimeHandle::default();
    assert!(!runtime.is_ready());
    assert!(runtime.mark_application_prepared());
    assert!(!runtime.is_ready());
    assert!(runtime.mark_running());
    assert!(runtime.is_ready());
}

#[test]
fn capability_payload_is_versioned_without_internal_details() {
    let ready = serde_json::to_value(BrowserCapability::Ready {
        engine_version: CEF_VERSION.to_string(),
    })
    .expect("serialize capability");
    let hidden = serde_json::to_value(BrowserCapability::Hidden).expect("serialize hidden");

    assert_eq!(
        ready,
        serde_json::json!({
            "status": "ready",
            "engineVersion": "150.0.0+150.0.10",
        })
    );
    assert_eq!(hidden, serde_json::json!({ "status": "hidden" }));
}
