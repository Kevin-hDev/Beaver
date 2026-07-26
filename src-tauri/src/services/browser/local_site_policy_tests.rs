use super::local_site_policy::{
    clean_local_title, is_allowed_local_url, is_internal_page, is_internal_port, looks_like_html,
};

#[test]
fn accepts_only_explicit_local_http_hosts() {
    for raw in [
        "http://localhost:3000/",
        "https://127.0.0.1:8443/",
        "http://[::1]:8080/",
    ] {
        assert!(is_allowed_local_url(&url::Url::parse(raw).unwrap()));
    }
    for raw in [
        "http://example.com/",
        "file:///tmp/test",
        "http://user:pass@localhost/",
        "http://127.0.0.2/",
    ] {
        assert!(!is_allowed_local_url(&url::Url::parse(raw).unwrap()));
    }
}

#[test]
fn recognizes_html_and_internal_services() {
    assert!(looks_like_html(Some("text/html; charset=utf-8"), "hello"));
    assert!(looks_like_html(None, "<!doctype html><title>Site</title>"));
    assert!(!looks_like_html(Some("application/json"), "{\"ok\":true}"));
    assert!(is_internal_page("<title>SearXNG</title>"));
    assert!(is_internal_page("<meta name='app' content='CL-GO Tauri'>"));
    assert!(is_internal_page("<meta name='app' content='Beaver Tauri'>"));
    assert!(is_internal_page("<title>Beaver Forecast Runtime</title>"));
    assert!(!is_internal_page("<title>Mon projet</title>"));
    assert!(is_internal_port(5_173));
    assert!(is_internal_port(11_434));
    assert!(is_internal_port(12_050));
    assert!(!is_internal_port(3_000));
}

#[test]
fn cleans_and_bounds_external_titles() {
    let title = clean_local_title(Some(format!(" <b>{}</b>\n", "é".repeat(100))), 3_000);
    assert_eq!(title.chars().count(), 80);
    assert!(!title.contains(['<', '>', '\n']));
    assert_eq!(clean_local_title(None, 3_001), "localhost:3001");
}
