use super::*;

fn valid_topic(id: &str) -> String {
    format!(
        "---\n\
         id: {id}\n\
         scope: global\n\
         type: preference\n\
         status: confirmed\n\
         title: Boutons compacts\n\
         summary: Kevin préfère les boutons compacts.\n\
         created_at: 2026-07-24T20:00:00Z\n\
         updated_at: 2026-07-24T20:00:00Z\n\
         tags: [ui, boutons]\n\
         source: user\n\
         session_id: 019f951b-38a1-7882-bf2f-0784e266c911\n\
         ---\n\
         # Boutons compacts\n\nPréférence confirmée."
    )
}

#[test]
fn parses_a_valid_topic() {
    let id = uuid::Uuid::new_v4().to_string();
    let path = std::path::PathBuf::from(format!("{id}.md"));

    let parsed = parse(&valid_topic(&id), &path, "global").unwrap();

    assert_eq!(parsed.topic.id, id);
    assert_eq!(parsed.topic.tags, ["ui", "boutons"]);
    assert_eq!(parsed.topic.memory_type, "preference");
}

#[test]
fn rejects_mismatched_id_and_scope() {
    let id = uuid::Uuid::new_v4().to_string();
    let other = uuid::Uuid::new_v4().to_string();
    assert!(parse(
        &valid_topic(&id),
        std::path::Path::new(&format!("{other}.md")),
        "global"
    )
    .is_err());
    assert!(parse(
        &valid_topic(&id),
        std::path::Path::new(&format!("{id}.md")),
        "project"
    )
    .is_err());
}

#[test]
fn rejects_common_secret_shapes() {
    let id = uuid::Uuid::new_v4().to_string();
    let content = format!("{}\npassword = super-secret-value", valid_topic(&id));

    let error = parse(
        &content,
        std::path::Path::new(&format!("{id}.md")),
        "global",
    )
    .unwrap_err();

    assert!(error.contains("sensibles"));
}

#[test]
fn rejects_json_web_tokens() {
    let id = uuid::Uuid::new_v4().to_string();
    let content = format!(
        "{}\n{}",
        valid_topic(&id),
        "eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.signature_value"
    );

    let error = parse(
        &content,
        std::path::Path::new(&format!("{id}.md")),
        "global",
    )
    .unwrap_err();

    assert!(error.contains("sensibles"));
}

#[test]
fn rejects_unbounded_tags() {
    let id = uuid::Uuid::new_v4().to_string();
    let content = valid_topic(&id).replace(
        "tags: [ui, boutons]",
        "tags: [a, b, c, d, e, f, g, h, i]",
    );

    assert!(parse(
        &content,
        std::path::Path::new(&format!("{id}.md")),
        "global"
    )
    .is_err());
}

#[test]
fn invalid_status_error_lists_the_allowed_values() {
    let id = uuid::Uuid::new_v4().to_string();
    let content = valid_topic(&id).replace("status: confirmed", "status: active");

    let error = parse(
        &content,
        std::path::Path::new(&format!("{id}.md")),
        "global",
    )
    .unwrap_err();

    assert!(error.contains("confirmed, inferred, stale, archived"));
}
