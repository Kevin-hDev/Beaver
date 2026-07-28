use super::source_validation::{git, npm};

#[test]
fn git_sources_accept_secure_https_ssh_and_optional_refs() {
    let https = git("https://github.com/example/beaver-extension.git#v1.2.0").unwrap();
    assert_eq!(
        https.clone_url,
        "https://github.com/example/beaver-extension.git"
    );
    assert_eq!(https.reference.as_deref(), Some("v1.2.0"));

    assert!(git("ssh://git@github.com/example/beaver-extension.git").is_ok());
    assert!(git("git@github.com:example/beaver-extension.git").is_ok());
    assert!(git(&format!(
        "https://github.com/example/beaver-extension.git#{}",
        "a".repeat(40)
    ))
    .is_ok());
}

#[test]
fn git_sources_reject_insecure_or_credential_bearing_urls() {
    assert!(git("http://github.com/example/extension.git").is_err());
    assert!(git("https://token@github.com/example/extension.git").is_err());
    assert!(git("https://github.com/example/extension.git?token=secret").is_err());
    assert!(git("https://github.com/example/extension.git#main#other").is_err());
    assert!(git("https://github.com/example/extension.git#../../main").is_err());
    assert!(git("--upload-pack=payload").is_err());
}

#[test]
fn npm_sources_accept_names_scopes_versions_and_tags() {
    let scoped = npm("@beaver/search-extension@2.1.0-beta.1").unwrap();
    assert_eq!(scoped.package_name, "@beaver/search-extension");
    assert!(npm("beaver-extension@latest").is_ok());
    assert!(npm("beaver-extension").is_ok());
}

#[test]
fn npm_sources_reject_ranges_paths_urls_and_option_injection() {
    assert!(npm("beaver-extension@^2.0.0").is_err());
    assert!(npm("../extension").is_err());
    assert!(npm("https://registry.example/extension.tgz").is_err());
    assert!(npm("--global").is_err());
    assert!(npm("@scope/package@").is_err());
}
