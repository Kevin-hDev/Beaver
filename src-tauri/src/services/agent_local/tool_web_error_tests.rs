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
    let config = search("Aucun provider configuré".to_string());
    let limited = search("Brave: limite de requêtes atteinte (HTTP 429)".to_string());
    let timeout = search("SearXNG: timeout".to_string());

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
    let result = search(
        "Brave: authentification refusée; Exa: limite de requêtes atteinte (HTTP 429)".to_string(),
    );

    let error = result.error.unwrap();
    assert_eq!(error.code.as_ref(), "web_search_rate_limited");
    assert!(error.retryable);
}

#[test]
fn searxng_machine_codes_keep_local_runtime_errors_translatable() {
    let runtime = search(crate::services::searxng::error_codes::RUNTIME_UNAVAILABLE.to_string());
    let cancelled = search(crate::services::searxng::error_codes::SHUTTING_DOWN.to_string());

    let runtime_error = runtime.error.unwrap();
    assert_eq!(runtime_error.code.as_ref(), "web_search_runtime_unavailable");
    assert_eq!(runtime_error.category, ToolErrorCategory::Unavailable);
    assert!(runtime_error.retryable);
    assert_eq!(
        cancelled.error.unwrap().category,
        ToolErrorCategory::Cancelled
    );
}
