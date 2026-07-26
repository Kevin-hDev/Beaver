use super::brand::{
    directory_change_commit_message, subagent_change_commit_message, user_agent,
    DIRECTORY_BASELINE_COMMIT_MESSAGE, DISPLAY_NAME, GIT_AUTHOR_EMAIL_CONFIG,
    GIT_AUTHOR_NAME_CONFIG, MCP_CLIENT_NAME, USER_AGENT_PRODUCT,
};

#[test]
fn public_brand_is_beaver() {
    assert_eq!(DISPLAY_NAME, "Beaver");
    assert_eq!(MCP_CLIENT_NAME, "Beaver");
    assert_eq!(USER_AGENT_PRODUCT, "Beaver");
}

#[test]
fn public_brand_excludes_legacy_and_abandoned_names() {
    for value in [DISPLAY_NAME, MCP_CLIENT_NAME, USER_AGENT_PRODUCT] {
        let normalized = value.to_ascii_lowercase();
        assert!(!normalized.contains("beavry"));
        assert!(!normalized.contains(&["cl", "go", "dash"].join("-")));
        assert!(!normalized.contains(&["cl", "go"].join("-")));
    }
}

#[test]
fn user_agent_uses_the_public_product_and_package_version() {
    assert_eq!(
        user_agent(),
        format!("Beaver/{}", env!("CARGO_PKG_VERSION"))
    );
}

#[test]
fn generated_git_identity_uses_the_public_brand() {
    assert_eq!(GIT_AUTHOR_NAME_CONFIG, "user.name=Beaver");
    assert_eq!(GIT_AUTHOR_EMAIL_CONFIG, "user.email=beaver@local");
    assert_eq!(
        DIRECTORY_BASELINE_COMMIT_MESSAGE,
        "Beaver directory baseline"
    );
}

#[test]
fn generated_git_messages_use_the_public_brand() {
    assert_eq!(
        directory_change_commit_message("change-id"),
        "Beaver temporary directory change\n\nBeaver-Subagent-Change: change-id"
    );
    assert_eq!(
        subagent_change_commit_message("change-id"),
        "Beaver temporary subagent change\n\nBeaver-Subagent-Change: change-id"
    );
}
