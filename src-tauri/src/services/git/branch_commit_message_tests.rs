use super::branch_commit;

#[test]
fn sanitize_description_none_stays_none() {
    assert_eq!(branch_commit::sanitize_description(None), Ok(None));
}

#[test]
fn sanitize_description_empty_becomes_none() {
    assert_eq!(
        branch_commit::sanitize_description(Some("   ".to_string())),
        Ok(None)
    );
}

#[test]
fn sanitize_description_normalizes_crlf() {
    let result = branch_commit::sanitize_description(Some("a\r\nb\rc".to_string()))
        .unwrap()
        .unwrap();
    assert_eq!(result, "a\nb\nc");
}

#[test]
fn sanitize_description_rejects_null_byte() {
    assert!(branch_commit::sanitize_description(Some("abc\0def".to_string())).is_err());
}

#[test]
fn sanitize_description_rejects_control_chars() {
    assert!(branch_commit::sanitize_description(Some("bad\x01".to_string())).is_err());
    assert!(branch_commit::sanitize_description(Some("bad\x1F".to_string())).is_err());
}

#[test]
fn sanitize_description_allows_newline_and_tab() {
    let result = branch_commit::sanitize_description(Some("line1\n\tindented".to_string()))
        .unwrap()
        .unwrap();
    assert_eq!(result, "line1\n\tindented");
}

#[test]
fn sanitize_description_rejects_overlong() {
    let too_long = "a".repeat(2001);
    assert!(branch_commit::sanitize_description(Some(too_long)).is_err());
}

#[test]
fn sanitize_description_accepts_at_max_length() {
    let exact = "a".repeat(2000);
    let result = branch_commit::sanitize_description(Some(exact))
        .unwrap()
        .unwrap();
    assert_eq!(result.chars().count(), 2000);
}

#[test]
fn sanitize_description_trims_whitespace() {
    let result = branch_commit::sanitize_description(Some("  hello  ".to_string()))
        .unwrap()
        .unwrap();
    assert_eq!(result, "hello");
}

#[test]
fn build_commit_message_without_description() {
    let msg = branch_commit::build_commit_message("feature", None);
    assert_eq!(msg, "WIP: save changes before switching to feature");
}

#[test]
fn build_commit_message_with_description() {
    let msg = branch_commit::build_commit_message("feature", Some("détails du commit"));
    assert!(msg.contains("WIP: save changes before switching to feature"));
    assert!(msg.contains("\n\ndétails du commit"));
}

#[test]
fn build_commit_message_with_empty_description_omits_body() {
    let msg = branch_commit::build_commit_message("dev", Some(""));
    assert_eq!(msg, "WIP: save changes before switching to dev");
}
