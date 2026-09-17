use super::*;

#[test]
fn requested_priorities_are_unique_and_bounded() {
    let ids = (0..MAX_USER_PRIORITY_PLUGINS)
        .map(|index| format!("example.plugin{index}"))
        .collect::<Vec<_>>();
    assert!(validate_requested(&ids).is_ok());

    let mut duplicate = ids.clone();
    duplicate[1] = duplicate[0].clone();
    assert!(validate_requested(&duplicate).is_err());

    let mut too_many = ids;
    too_many.push("example.extra".to_string());
    assert!(validate_requested(&too_many).is_err());
}

#[test]
fn invalid_identifiers_are_rejected() {
    assert!(validate_requested(&["bad id".to_string()]).is_err());
}
