#![cfg(test)]

use super::web_search_status::format_web_search_status;

#[test]
fn no_provider_returns_failure_warning() {
    let section = format_web_search_status(false, false, false, false);
    assert!(
        section.contains("## Web search status"),
        "should always include the section header"
    );
    assert!(
        section.contains("WILL FAIL"),
        "should warn the tool will fail when no provider is configured"
    );
    assert!(
        section.contains("Settings → API keys"),
        "should point the user to the settings panel"
    );
}

#[test]
fn brave_provider_is_listed_when_active() {
    let section = format_web_search_status(true, false, false, false);
    assert!(section.contains("Active providers: Brave."));
    assert!(!section.contains("Exa"));
    assert!(!section.contains("SearXNG"));
}

#[test]
fn searxng_is_listed_with_local_label() {
    let section = format_web_search_status(false, false, false, true);
    assert!(
        section.contains("SearXNG (local fallback)"),
        "SearXNG must be advertised as the local fallback"
    );
}

#[test]
fn all_providers_are_joined() {
    let section = format_web_search_status(true, true, true, true);
    assert!(section.contains("Brave"));
    assert!(section.contains("Exa"));
    assert!(section.contains("Firecrawl"));
    assert!(section.contains("SearXNG (local fallback)"));
    assert!(
        section.contains("Provider is selected automatically"),
        "should remind the LLM it cannot pick a provider"
    );
}

#[test]
fn firecrawl_alone_is_listed() {
    let section = format_web_search_status(false, false, true, false);
    assert!(section.contains("Active providers: Firecrawl."));
}
