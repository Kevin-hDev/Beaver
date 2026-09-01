use super::caller::{authorize, TerminalOwner};

#[test]
fn only_the_main_webview_is_authorized() {
    assert!(authorize("main").is_ok());
    for label in [
        "mascot",
        "forecast-workbench",
        "forecast-docs",
        "MAIN",
        "",
        "main\n",
    ] {
        assert_eq!(authorize(label).unwrap_err(), "terminal-not-authorized");
    }
}

#[test]
fn the_test_owner_fixture_rejects_an_oversized_label() {
    assert_eq!(
        TerminalOwner::for_test(&"x".repeat(129)).unwrap_err(),
        "terminal-not-authorized"
    );
}
