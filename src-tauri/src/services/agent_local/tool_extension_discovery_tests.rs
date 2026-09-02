use super::*;

#[test]
fn report_distinguishes_loaded_and_omitted_plugins() {
    let result = discovery_result(vec![
        DiscoveryLine {
            plugin_id: "beaver.office.documents".to_string(),
            plugin_name: "Documents".to_string(),
            status: DiscoveryStatus::Loaded,
        },
        DiscoveryLine {
            plugin_id: "example.large".to_string(),
            plugin_name: "Large".to_string(),
            status: DiscoveryStatus::ProviderLimit,
        },
        DiscoveryLine {
            plugin_id: "example.overflow".to_string(),
            plugin_name: "Overflow".to_string(),
            status: DiscoveryStatus::DiscoveryLimit,
        },
        DiscoveryLine {
            plugin_id: "example.unavailable".to_string(),
            plugin_name: "Unavailable".to_string(),
            status: DiscoveryStatus::Unavailable,
        },
    ]);

    let output = &result.content;
    assert!(output.contains("Documents : outils chargés"));
    assert!(output.contains("Large : non chargé"));
    assert!(output.contains("limite de plugins découverts"));
    assert!(output.contains("outils indisponibles dans cette requête"));
    assert_eq!(
        result.status,
        super::super::tool_result_contract::ToolResultStatus::Partial
    );
    assert_eq!(result.warnings.len(), 1);
}

#[test]
fn fully_loaded_discovery_is_a_clean_success() {
    let result = discovery_result(vec![DiscoveryLine {
        plugin_id: "beaver.office.documents".to_string(),
        plugin_name: "Documents".to_string(),
        status: DiscoveryStatus::Loaded,
    }]);

    assert_eq!(
        result.status,
        super::super::tool_result_contract::ToolResultStatus::Success
    );
    assert!(result.warnings.is_empty());
}

#[test]
fn discovered_ids_are_unique_and_bounded() {
    let mut ids = vec!["example.one".to_string()];
    assert!(push_unique(&mut ids, "example.one"));
    assert!(push_unique(&mut ids, "example.two"));

    assert_eq!(ids, vec!["example.one", "example.two"]);
}

#[test]
fn discovery_limit_is_reported_separately() {
    let mut ids = (0..crate::services::extensions::MAX_DISCOVERED_PLUGINS)
        .map(|index| format!("example.plugin{index}"))
        .collect::<Vec<_>>();

    assert!(!push_unique(&mut ids, "example.overflow"));
}

#[test]
fn a_plugin_without_tools_is_never_reported_as_available() {
    assert_eq!(
        existing_status(0, true),
        Some(DiscoveryStatus::NoTools)
    );
}

#[test]
fn discovery_statuses_use_only_the_three_contractual_search_reasons() {
    for status in [
        DiscoveryStatus::Loaded,
        DiscoveryStatus::AlreadyAvailable,
        DiscoveryStatus::NoTools,
        DiscoveryStatus::Unavailable,
    ] {
        assert_eq!(
            status.diagnostic_reason(),
            super::super::stream_diagnostics::ExtensionDiagnosticReason::DiscoveryResult
        );
    }
    assert_eq!(
        DiscoveryStatus::ProviderLimit.diagnostic_reason(),
        super::super::stream_diagnostics::ExtensionDiagnosticReason::ProviderCapacity
    );
    assert_eq!(
        DiscoveryStatus::DiscoveryLimit.diagnostic_reason(),
        super::super::stream_diagnostics::ExtensionDiagnosticReason::GlobalCapacity
    );
}

#[tokio::test]
async fn every_search_has_a_distinct_diagnostic_id_without_query_text() {
    let session = super::super::session_store::create_full(
        "Discovery diagnostics",
        "qwen-max",
        "qwen",
        false,
        None,
    )
    .await
    .unwrap();
    let request_id =
        super::super::stream_diagnostics::start_request(&session.id, 1).await;
    let query = serde_json::json!({
        "query": "SENTINEL_USER_MESSAGE_WITH_NO_PLUGIN_MATCH_92c879e8"
    });

    execute(&query, &session.id, &request_id).await;
    execute(&query, &session.id, &request_id).await;
    execute(&serde_json::json!({"query": 42}), &session.id, &request_id).await;
    execute(
        &serde_json::json!({
            "query": "x".repeat(crate::services::extensions::MAX_SEARCH_QUERY_CHARS + 1)
        }),
        &session.id,
        &request_id,
    )
    .await;

    let stored = super::super::session_store::get(&session.id).await.unwrap();
    let run = stored
        .diagnostic_runs
        .iter()
        .find(|run| run.request_id == request_id)
        .unwrap();
    let searches = run
        .events
        .iter()
        .filter_map(|event| event.extension.as_ref())
        .filter(|diagnostic| diagnostic.origin == "extension_tool_search")
        .collect::<Vec<_>>();
    assert_eq!(searches.len(), 4);
    let first = searches[0].correlation_id.as_deref().unwrap();
    let second = searches[1].correlation_id.as_deref().unwrap();
    assert!(uuid::Uuid::parse_str(first).is_ok());
    assert!(uuid::Uuid::parse_str(second).is_ok());
    assert_ne!(first, second);
    assert_eq!(
        searches
            .iter()
            .filter_map(|diagnostic| diagnostic.correlation_id.as_deref())
            .collect::<std::collections::HashSet<_>>()
            .len(),
        4
    );
    assert!(!serde_json::to_string(run).unwrap().contains("SENTINEL_USER_MESSAGE"));
    super::super::session_store::delete_one(&session.id)
        .await
        .unwrap();
}

#[tokio::test]
async fn unavailable_discovery_state_still_records_the_search_correlation() {
    let session = super::super::session_store::create_full(
        "Unavailable discovery diagnostics",
        "qwen-max",
        "qwen",
        false,
        None,
    )
    .await
    .unwrap();
    let request_id = super::super::stream_diagnostics::start_request(&session.id, 1).await;
    let search_id = uuid::Uuid::from_u128(42).to_string();
    let matches = vec![crate::services::extensions::PluginMatch {
        extension_id: "example.documents".to_string(),
        extension_name: "Documents".to_string(),
        score: 1,
    }];

    let result = execute_matches(
        &session.id,
        &request_id,
        &search_id,
        &matches,
    )
    .await;

    assert_eq!(
        result.error.as_ref().map(|error| error.code.as_ref()),
        Some("plugin_search_unavailable")
    );
    let stored = super::super::session_store::get(&session.id).await.unwrap();
    let search = stored
        .diagnostic_runs
        .iter()
        .find(|run| run.request_id == request_id)
        .and_then(|run| run.events.iter().find_map(|event| event.extension.as_ref()))
        .unwrap();
    assert_eq!(search.correlation_id.as_deref(), Some(search_id.as_str()));
    assert_eq!(search.reason, "discovery_result");
    super::super::session_store::delete_one(&session.id).await.unwrap();
}
