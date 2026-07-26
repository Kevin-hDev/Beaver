use super::navigation_target::NavigationTarget;

#[test]
fn explicit_navigation_is_consumed_once() {
    let mut target = NavigationTarget::default();
    target.request("https://example.com/");
    assert_eq!(
        target.take_pending().as_deref(),
        Some("https://example.com/")
    );
    assert!(target.take_pending().is_none());
}

#[test]
fn observed_back_navigation_never_becomes_an_explicit_request() {
    let mut target = NavigationTarget::default();
    target.observe("http://localhost:43123/");
    assert!(target.take_pending().is_none());
}
