use super::{decision, discover_with_refresh, ids, indexed_active_record_is_available, Decision};
use crate::services::agent_local::{
    extension_session_plugins::refresh_active_with_catalog,
    extension_session_state::ExtensionSessionState, extension_tool_selection::PluginDescriptor,
};
use crate::services::extensions::CatalogSnapshot;
use serde_json::json;

#[tokio::test]
async fn invalid_inspection_uses_the_contract_error_code() {
    let result = super::execute(&json!({"ids": []}), "unused", None).await;

    assert_eq!(
        result.error.as_ref().map(|error| error.code.as_ref()),
        Some(crate::services::extensions::error_codes::INSPECTION_INVALID)
    );
}

#[tokio::test]
async fn unavailable_inspection_uses_the_contract_error_code() {
    let result = super::execute(&json!({"ids": ["example.a"]}), "missing-session", None).await;

    assert_eq!(
        result.error.as_ref().map(|error| error.code.as_ref()),
        Some(crate::services::extensions::error_codes::INSPECTION_UNAVAILABLE)
    );
}

#[test]
fn accepts_one_to_four_unique_exact_identifiers_in_order() {
    assert_eq!(
        ids(&json!({"ids":["example.a","example.b","example.c","example.d"]})).unwrap(),
        vec!["example.a", "example.b", "example.c", "example.d"]
    );
}

#[test]
fn rejects_duplicate_and_more_than_four_identifiers() {
    assert!(ids(&json!({"ids":["example.a","example.a"]})).is_err());
    assert!(
        ids(&json!({"ids":["example.a","example.b","example.c","example.d","example.e"]})).is_err()
    );
}

#[test]
fn active_approved_records_missing_from_the_index_are_unavailable() {
    assert!(!indexed_active_record_is_available(
        Some((true, true)),
        false
    ));
    assert!(indexed_active_record_is_available(Some((true, true)), true));
    assert!(indexed_active_record_is_available(
        Some((false, true)),
        false
    ));
}

#[test]
fn classifies_every_inspection_status_without_mutation() {
    let cases = [
        (decision(None, None, false), Decision::Unknown, false),
        (
            decision(Some((false, true)), None, false),
            Decision::Inactive,
            false,
        ),
        (
            decision(Some((true, false)), None, false),
            Decision::Unapproved,
            false,
        ),
        (
            decision(Some((true, true)), Some(false), true),
            Decision::AlreadyAvailable,
            true,
        ),
        (
            decision(Some((true, true)), Some(false), false),
            Decision::Loaded,
            true,
        ),
        (
            decision(Some((true, true)), Some(true), false),
            Decision::NoTools,
            true,
        ),
    ];
    for (actual, expected, admissible) in cases {
        assert_eq!(actual, expected);
        assert_eq!(actual.admissible(), admissible);
    }
}

fn state(capacity: usize, tools: usize) -> ExtensionSessionState {
    ExtensionSessionState {
        discovered_plugin_ids: Vec::new(),
        epoch: None,
        plugin_tool_capacity: capacity,
        plugin_descriptors: vec![PluginDescriptor {
            id: "example.a".into(),
            tool_count: tools,
            definition_count: tools,
        }],
        active_plugin_ids: Vec::new(),
    }
}

fn catalog() -> CatalogSnapshot {
    CatalogSnapshot {
        ordered_plugin_ids: vec!["example.a".into()],
        capacity_plugin_ids: vec!["example.a".into()],
        ..Default::default()
    }
}

#[test]
fn capacity_admits_plugin_and_activates_it() {
    let catalog = catalog();
    let mut state = state(1, 1);

    assert!(discover_with_refresh(&mut state, "example.a", |state| {
        refresh_active_with_catalog(state, false, &catalog)
    }));
    assert_eq!(state.discovered_plugin_ids, ["example.a"]);
    assert_eq!(state.active_plugin_ids, ["example.a"]);
}

#[test]
fn capacity_overflow_rolls_back_the_discovery() {
    let catalog = catalog();
    let mut state = state(0, 1);

    assert!(!discover_with_refresh(&mut state, "example.a", |state| {
        refresh_active_with_catalog(state, false, &catalog)
    }));
    assert!(state.discovered_plugin_ids.is_empty());
}

#[test]
fn no_tools_is_admissible_without_capacity() {
    let catalog = catalog();
    let mut state = state(0, 0);

    assert!(discover_with_refresh(&mut state, "example.a", |state| {
        refresh_active_with_catalog(state, false, &catalog)
    }));
    assert_eq!(state.discovered_plugin_ids, ["example.a"]);
}

#[test]
fn inspection_records_discovery_when_plugin_tools_were_already_active() {
    let catalog = catalog();
    let mut state = state(1, 1);
    state.active_plugin_ids.push("example.a".into());

    assert!(discover_with_refresh(&mut state, "example.a", |state| {
        refresh_active_with_catalog(state, false, &catalog)
    }));
    assert_eq!(state.discovered_plugin_ids, ["example.a"]);
}

#[tokio::test]
async fn inspection_event_is_bounded_and_refresh_correlates_its_uuid() {
    let session = super::super::session_store::create_full(
        "Inspection diagnostics",
        "qwen",
        "qwen",
        false,
        None,
    )
    .await
    .unwrap();
    let request_id = super::super::stream_diagnostics::start_request(&session.id, 1).await;
    let result = super::execute(
        &json!({"ids": ["example.a"]}),
        &session.id,
        Some(&request_id),
    )
    .await;

    assert!(!result.is_error);
    let stored = super::super::session_store::get(&session.id).await.unwrap();
    let inspection = stored
        .diagnostic_runs
        .iter()
        .find(|run| run.request_id == request_id)
        .unwrap()
        .events
        .iter()
        .find_map(|event| event.extension.as_ref())
        .unwrap();
    let correlation_id = inspection.correlation_id.clone().unwrap();
    assert!(uuid::Uuid::parse_str(&correlation_id).is_ok());
    assert!(inspection.plugin_count <= 4);
    assert_eq!(inspection.reason, "inspection_result");
    let pending =
        super::super::stream_diagnostics::pending_extension_inspections(&session.id, &request_id)
            .await;
    super::super::stream_diagnostics::record_extension_refreshes(
        &session.id,
        &request_id,
        pending,
        &["tool.a".to_string()],
        "qwen",
        &[],
    )
    .await;
    let stored = super::super::session_store::get(&session.id).await.unwrap();
    let refreshed = stored
        .diagnostic_runs
        .iter()
        .find(|run| run.request_id == request_id)
        .unwrap()
        .events
        .iter()
        .filter_map(|event| event.extension.as_ref())
        .find(|diagnostic| diagnostic.origin == "extension_tools_refreshed")
        .unwrap();
    assert_eq!(refreshed.related_inspection_ids, [correlation_id]);
    super::super::session_store::delete_one(&session.id)
        .await
        .unwrap();
}

#[tokio::test]
async fn invalid_inspection_records_no_untrusted_identifier() {
    let session =
        super::super::session_store::create_full("Invalid inspection", "qwen", "qwen", false, None)
            .await
            .unwrap();
    let request_id = super::super::stream_diagnostics::start_request(&session.id, 1).await;
    let result = super::execute(
        &json!({"ids": ["not valid!"]}),
        &session.id,
        Some(&request_id),
    )
    .await;

    assert!(result.is_error);
    let stored = super::super::session_store::get(&session.id).await.unwrap();
    let inspection = stored
        .diagnostic_runs
        .iter()
        .find(|run| run.request_id == request_id)
        .unwrap()
        .events
        .iter()
        .find_map(|event| event.extension.as_ref())
        .unwrap();
    assert_eq!(inspection.plugin_count, 0);
    assert!(inspection.plugin_ids.is_empty());
    super::super::session_store::delete_one(&session.id)
        .await
        .unwrap();
}
