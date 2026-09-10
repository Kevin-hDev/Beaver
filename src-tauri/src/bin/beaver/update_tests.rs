#[test]
fn plus_recent_detecte() {
    assert!(cl_go_dash_lib::cli_support::version_gt("1.3.0", "1.2.1"));
    assert!(!cl_go_dash_lib::cli_support::version_gt("1.2.1", "1.2.1"));
}
