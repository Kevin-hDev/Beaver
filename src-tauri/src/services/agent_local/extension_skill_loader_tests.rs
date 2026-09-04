#[test]
fn maps_extension_loader_failures_to_the_existing_generic_skill_errors() {
    use super::extension_skill_loader::map_error;
    use super::tool_skill_loader::SkillLoadError;
    use crate::services::extensions::ResourceLoadError;

    assert_eq!(map_error(ResourceLoadError::InvalidId), SkillLoadError::InvalidId);
    assert_eq!(map_error(ResourceLoadError::TooLarge), SkillLoadError::Unavailable);
    assert_eq!(map_error(ResourceLoadError::NotFound), SkillLoadError::NotFound);
    assert_eq!(map_error(ResourceLoadError::Unavailable), SkillLoadError::Unavailable);
}

#[test]
fn enriched_extension_skill_is_bounded_after_provenance_is_added() {
    let prefix = "Skill source: example.extension\n\n";
    let exact_body = "x".repeat(super::skill_limits::MAX_RESOLVED_SKILL_BYTES - prefix.len());
    let exact = super::extension_skill_loader::enrich(
        "Guide".into(),
        "example.extension".into(),
        exact_body.into_bytes(),
    )
    .expect("exact resolved limit");
    assert_eq!(exact.content.len(), super::skill_limits::MAX_RESOLVED_SKILL_BYTES);

    let oversized_body = "x".repeat(
        super::skill_limits::MAX_RESOLVED_SKILL_BYTES - prefix.len() + 1,
    );
    assert!(matches!(
        super::extension_skill_loader::enrich(
            "Guide".into(),
            "example.extension".into(),
            oversized_body.into_bytes(),
        ),
        Err(super::tool_skill_loader::SkillLoadError::Unavailable)
    ));
}

#[tokio::test]
async fn ordinary_skill_ids_still_delegate_to_the_global_catalog() {
    let result = super::extension_skill_loader::load_skill_for_session(
        "local:skill:000000000000000000000000",
        "no-session-needed",
    )
    .await;

    assert!(matches!(result, Err(super::tool_skill_loader::SkillLoadError::NotFound)));
}
