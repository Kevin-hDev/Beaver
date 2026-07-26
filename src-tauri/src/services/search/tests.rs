use super::*;

#[test]
fn provider_order_keeps_brave_first_and_searxng_external() {
    assert_eq!(
        PROVIDER_ORDER,
        [
            SearchProvider::Brave,
            SearchProvider::Exa,
            SearchProvider::Firecrawl
        ]
    );
}

#[test]
fn failure_message_keeps_causes() {
    let msg = format_failures(&[
        "Brave: HTTP 429".to_string(),
        "SearXNG: timeout au démarrage".to_string(),
    ]);
    assert!(msg.contains("Brave: HTTP 429"));
    assert!(msg.contains("SearXNG: timeout au démarrage"));
}

#[tokio::test]
async fn configured_providers_fallback_until_success() {
    let mut calls = Vec::new();
    let (configured, failures, result) = try_configured_providers(
        "query",
        |_| true,
        |provider, _| {
            calls.push(provider);
            async move {
                match provider {
                    SearchProvider::Brave => Err("Brave: HTTP 429".to_string()),
                    SearchProvider::Exa => Ok(Vec::new()),
                    SearchProvider::Firecrawl => Ok(vec![SearchResult {
                        title: "ok".to_string(),
                        url: "https://example.com".to_string(),
                        snippet: "body".to_string(),
                    }]),
                }
            }
        },
    )
    .await;

    assert!(configured);
    assert_eq!(calls, PROVIDER_ORDER);
    assert!(failures
        .iter()
        .any(|failure| failure.contains("Brave: HTTP 429")));
    assert!(failures
        .iter()
        .any(|failure| failure.contains("Exa: résultat vide")));
    assert_eq!(result.unwrap().len(), 1);
}

#[tokio::test]
async fn no_configured_provider_skips_provider_calls() {
    let (configured, failures, result) =
        try_configured_providers("query", |_| false, |_, _| async { Ok(Vec::new()) }).await;

    assert!(!configured);
    assert!(failures.is_empty());
    assert!(result.is_none());
}
