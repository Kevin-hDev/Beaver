use super::runtime_handle::{BrowserCapability, BrowserRuntimeHandle, CEF_VERSION};
use super::runtime_integration::is_browser_ready_event;

#[test]
fn browser_initialization_accepts_only_the_ready_event() {
    assert!(is_browser_ready_event(&tauri::RunEvent::Ready));
    assert!(!is_browser_ready_event(&tauri::RunEvent::Resumed));
    assert!(!is_browser_ready_event(&tauri::RunEvent::MainEventsCleared));
}

#[test]
fn production_runtime_applies_the_ready_event_gate() {
    let source = include_str!("runtime_integration.rs");

    assert!(source.contains("if !is_browser_ready_event(event)"));
}

#[test]
fn capability_events_apply_the_same_security_gate_as_commands() {
    let source = include_str!("cef_cookie_gate.rs");

    assert!(source.contains("super::cef_runtime_policy::emit_capability(&self.app, &self.runtime)"));
    assert!(!source.contains("self.runtime.capability()"));
}

#[test]
fn runtime_becomes_ready_only_after_ordered_bootstrap() {
    let invalid = BrowserRuntimeHandle::default();
    assert!(invalid.mark_application_prepared());
    assert!(!invalid.mark_running());
    assert_eq!(
        invalid.capability(),
        BrowserCapability::Unavailable {
            restart_recommended: true,
        }
    );

    let runtime = BrowserRuntimeHandle::default();

    assert_eq!(
        runtime.capability(),
        BrowserCapability::Unavailable {
            restart_recommended: false,
        }
    );
    assert!(runtime.mark_application_prepared());
    assert!(runtime.mark_supervised());
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
    assert_eq!(
        clone.capability(),
        BrowserCapability::Unavailable {
            restart_recommended: true,
        }
    );
}

#[test]
fn native_surface_is_allowed_only_after_the_security_gate() {
    let runtime = BrowserRuntimeHandle::default();
    assert!(!runtime.is_ready());
    assert!(runtime.mark_application_prepared());
    assert!(!runtime.is_ready());
    assert!(runtime.mark_supervised());
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
    let unavailable = serde_json::to_value(BrowserCapability::Unavailable {
        restart_recommended: true,
    })
    .expect("serialize unavailable");

    assert_eq!(
        ready,
        serde_json::json!({
            "status": "ready",
            "engineVersion": "150.0.0+150.0.10",
        })
    );
    assert_eq!(hidden, serde_json::json!({ "status": "hidden" }));
    assert_eq!(
        unavailable,
        serde_json::json!({
            "status": "unavailable",
            "restartRecommended": true,
        })
    );
}
