use super::*;

#[test]
fn web_fetch_http_statuses_are_actionable() {
    let missing = fetch("HTTP 404".to_string());
    let limited = fetch("HTTP 429".to_string());
    let server = fetch("HTTP 503".to_string());

    assert_eq!(missing.error.unwrap().category, ToolErrorCategory::NotFound);
    assert_eq!(
        limited.error.unwrap().code.as_ref(),
        "web_fetch_rate_limited"
    );
    assert!(server.error.unwrap().retryable);
}

#[test]
fn blocked_and_invalid_urls_are_not_transport_failures() {
    let invalid = fetch("URL invalide".to_string());
    let blocked = fetch("adresse privée bloquée".to_string());

    assert_eq!(invalid.error.unwrap().category, ToolErrorCategory::Validation);
    assert_eq!(blocked.error.unwrap().category, ToolErrorCategory::Permission);
}

#[test]
fn search_configuration_rate_limit_and_timeout_are_distinct() {
    let config = search(unstructured("Aucun provider configuré"));
    let limited = search(unstructured("Brave: limite de requêtes atteinte (HTTP 429)"));
    let timeout = search(unstructured("SearXNG: timeout"));

    let config_error = config.error.unwrap();
    assert_eq!(config_error.code.as_ref(), "web_search_not_configured");
    assert!(config_error
        .hint
        .as_deref()
        .is_some_and(|hint| hint.contains("Configurer")));
    assert_eq!(
        limited.error.unwrap().code.as_ref(),
        "web_search_rate_limited"
    );
    assert_eq!(timeout.error.unwrap().category, ToolErrorCategory::Timeout);
}

#[test]
fn a_retryable_provider_failure_wins_over_an_auth_failure_from_another_provider() {
    let result = search(unstructured(
        "Brave: authentification refusée; Exa: limite de requêtes atteinte (HTTP 429)",
    ));

    let error = result.error.unwrap();
    assert_eq!(error.code.as_ref(), "web_search_rate_limited");
    assert!(error.retryable);
}

#[test]
fn searxng_machine_codes_keep_local_runtime_errors_translatable() {
    let runtime = search(crate::services::search::SearchFailure::searxng(
        crate::services::searxng::error_codes::RUNTIME_UNAVAILABLE,
    ));
    let cancelled = search(crate::services::search::SearchFailure::searxng(
        crate::services::searxng::error_codes::SHUTTING_DOWN,
    ));

    let runtime_error = runtime.error.unwrap();
    assert_eq!(runtime_error.code.as_ref(), "web_search_runtime_unavailable");
    assert_eq!(runtime_error.category, ToolErrorCategory::Unavailable);
    assert!(runtime_error.retryable);
    assert_eq!(
        cancelled.error.unwrap().category,
        ToolErrorCategory::Cancelled
    );
}

#[test]
fn every_declared_searxng_code_has_an_explicit_tool_classification() {
    use crate::services::searxng::error_codes;

    let expected = [
        (error_codes::APP_UNAVAILABLE, "web_search_runtime_unavailable"),
        (error_codes::BUNDLE_INVALID, "web_search_runtime_unavailable"),
        (error_codes::CONFIG_UNAVAILABLE, "web_search_runtime_unavailable"),
        (error_codes::LOG_UNAVAILABLE, "web_search_runtime_unavailable"),
        (error_codes::OPERATION_INTERRUPTED, "web_search_runtime_unavailable"),
        (error_codes::PROCESS_STATE_UNAVAILABLE, "web_search_runtime_unavailable"),
        (error_codes::RUNTIME_UNAVAILABLE, "web_search_runtime_unavailable"),
        (error_codes::SEARCH_FAILED, "web_search_unavailable"),
        (error_codes::SEARCH_INVALID_RESPONSE, "web_search_invalid_response"),
        (error_codes::SEARCH_RATE_LIMITED, "web_search_rate_limited"),
        (error_codes::SETTINGS_UNAVAILABLE, "web_search_runtime_unavailable"),
        (error_codes::SHUTTING_DOWN, "web_search_cancelled"),
        (error_codes::SOURCE_UNAVAILABLE, "web_search_runtime_unavailable"),
        (error_codes::START_FAILED, "web_search_runtime_unavailable"),
    ];
    assert_eq!(expected.len(), error_codes::ALL.len());
    for (code, expected_tool_code) in expected {
        let failure = crate::services::search::finish_search(
            false,
            false,
            Vec::new(),
            Err(code.to_string()),
        )
        .unwrap_err();
        let result = search(failure);
        assert_eq!(result.error.unwrap().code.as_ref(), expected_tool_code, "{code}");
    }
}

#[test]
fn configured_provider_failures_keep_their_tool_classification_when_searxng_is_down() {
    let cases = [
        (
            "Brave: authentification refusée",
            "web_search_auth_failed",
            ToolErrorCategory::Permission,
        ),
        (
            "Exa: limite de requêtes atteinte (HTTP 429)",
            "web_search_rate_limited",
            ToolErrorCategory::External,
        ),
    ];

    for (provider_failure, expected_code, expected_category) in cases {
        let failure = crate::services::search::finish_search(
            true,
            false,
            vec![provider_failure.to_string()],
            Err(crate::services::searxng::error_codes::RUNTIME_UNAVAILABLE.to_string()),
        )
        .unwrap_err();
        let error = search(failure).error.unwrap();

        assert_eq!(error.code.as_ref(), expected_code, "{provider_failure}");
        assert_eq!(error.category, expected_category, "{provider_failure}");
    }
}
