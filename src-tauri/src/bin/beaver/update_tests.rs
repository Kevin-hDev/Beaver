#[test]
fn plus_recent_detecte() {
    assert!(cl_go_dash_lib::cli_support::version_gt("1.3.0", "1.2.1"));
    assert!(!cl_go_dash_lib::cli_support::version_gt("1.2.1", "1.2.1"));
}

#[test]
fn detail_technique_est_borne_et_sans_controle() {
    let detail = format!("a\n\u{1b}[31m{}", "é".repeat(600));
    let safe = super::safe_detail(&detail);

    assert_eq!(safe.chars().count(), 512);
    assert!(!safe.chars().any(char::is_control));
    assert_eq!(safe.chars().nth(1), Some('?'));
    assert_eq!(safe.chars().nth(2), Some('?'));
}
