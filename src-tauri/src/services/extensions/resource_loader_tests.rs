use super::resource_loader::{parse_qualified_contribution_id, QualifiedContributionId};

#[test]
fn parses_exactly_three_qualified_segments_and_keeps_a_valid_double_dot_extension_id() {
    assert_eq!(
        parse_qualified_contribution_id("extension:a..b:guide"),
        Ok(QualifiedContributionId {
            extension_id: "a..b".to_string(),
            local_id: "guide".to_string(),
        })
    );
}

#[test]
fn rejects_missing_or_extra_qualified_segments() {
    for id in [
        "extension",
        "extension:a",
        "extension:a:guide:extra",
        "other:a:guide",
    ] {
        assert!(parse_qualified_contribution_id(id).is_err(), "{id}");
    }
}
