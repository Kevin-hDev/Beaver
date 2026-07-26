use super::url_policy::validate_browser_url;

#[test]
fn browser_url_accepts_http_https_and_localhost() {
    assert!(validate_browser_url("http://localhost:5173/").is_ok());
    assert!(validate_browser_url("https://example.com/path?q=ok").is_ok());
}

#[test]
fn browser_url_rejects_credentials_protocols_controls_and_oversize() {
    for invalid in [
        "https://user:secret@example.com/",
        "file:///tmp/private",
        "javascript:alert(1)",
        "https://example.com/\nnext",
        "https:///missing-host",
    ] {
        assert!(validate_browser_url(invalid).is_err(), "accepted {invalid}");
    }
    assert!(validate_browser_url(&format!("https://example.com/{}", "a".repeat(2_048))).is_err());
}
