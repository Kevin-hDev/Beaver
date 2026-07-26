use reqwest::header::{AUTHORIZATION, COOKIE, PROXY_AUTHORIZATION, USER_AGENT};

use super::*;

const VERSION: &str = "1.1.0";
const ASSET: &str = "Beaver_1.1.0_aarch64.dmg";

fn valid_asset_url() -> url::Url {
    url::Url::parse(&format!(
        "https://github.com/Kevin-hDev/Beaver/releases/download/v{VERSION}/{ASSET}"
    ))
    .expect("valid asset URL")
}

#[test]
fn bridge_source_is_exactly_beaver() {
    assert_eq!(UPDATE_SOURCE.repository, "Kevin-hDev/Beaver");
    assert_eq!(UPDATE_SOURCE.asset_prefix, "Beaver_");
    assert_eq!(UPDATE_SOURCE.release_product, "Beaver");
    assert_eq!(
        UPDATE_SOURCE
            .latest_release_url()
            .expect("constant URL")
            .as_str(),
        "https://api.github.com/repos/Kevin-hDev/Beaver/releases/latest"
    );
    assert!(strict_version_gt("1.1.0", "1.0.2"));
    assert!(!strict_version_gt("1.1.0-beta", "1.0.2"));
}

#[test]
fn latest_release_url_rejects_every_lookalike() {
    let invalid = [
        "http://api.github.com/repos/Kevin-hDev/Beaver/releases/latest",
        "https://api.github.com.evil.test/repos/Kevin-hDev/Beaver/releases/latest",
        "https://api.github.com:443/repos/Kevin-hDev/Beaver/releases/latest",
        "https://user@api.github.com/repos/Kevin-hDev/Beaver/releases/latest",
        "https://api.github.com/repos/Kevin-hDev/Beaver-copy/releases/latest",
        "https://api.github.com/repos/Kevin-hDev/Beaver/releases/../latest",
        "https://api.github.com/repos/Kevin-hDev/Beaver/releases/%2e%2e/latest",
        "https://api.github.com/repos/Kevin-hDev/Beaver/releases/latest?ref=main",
    ];

    for url in invalid {
        assert!(!UPDATE_SOURCE.is_latest_release_url(url), "{url}");
    }
}

#[test]
fn notes_url_uses_the_same_repository_and_tag() {
    let expected =
        "https://raw.githubusercontent.com/Kevin-hDev/Beaver/v1.1.0/app-release-notes.json";
    assert_eq!(
        UPDATE_SOURCE
            .release_notes_url(VERSION)
            .expect("notes URL")
            .as_str(),
        expected
    );
    assert!(UPDATE_SOURCE.is_release_notes_url(expected, VERSION));
    assert!(!UPDATE_SOURCE.is_release_notes_url(
        "https://raw.githubusercontent.com/Kevin-hDev/CL-GO-DASH/v1.1.0/app-release-notes.json",
        VERSION,
    ));
    assert!(UPDATE_SOURCE.release_notes_url("../1.1.0").is_none());
}

#[test]
fn manifest_url_uses_the_exact_release_and_tag() {
    let expected =
        "https://github.com/Kevin-hDev/Beaver/releases/download/v1.1.0/update-manifest.json";
    assert_eq!(
        UPDATE_SOURCE
            .manifest_url(VERSION)
            .expect("manifest URL")
            .as_str(),
        expected
    );
    assert!(UPDATE_SOURCE.is_manifest_url(expected, VERSION));
    assert!(!UPDATE_SOURCE.is_manifest_url(
        "https://github.com/Kevin-hDev/Beaver-copy/releases/download/v1.1.0/update-manifest.json",
        VERSION,
    ));
}

#[test]
fn asset_url_requires_exact_repository_tag_and_name() {
    let valid = valid_asset_url();
    let parsed = UPDATE_SOURCE
        .asset_reference(valid.as_str())
        .expect("trusted asset");
    assert_eq!(parsed.version, VERSION);
    assert_eq!(parsed.name, ASSET);

    let invalid = [
        "https://github.com/Kevin-hDev/CL-GO-DASH/releases/download/v1.1.0/Beaver_1.1.0_aarch64.dmg",
        "https://github.com/Kevin-hDev/Beaver/releases/download/v1.1.1/Beaver_1.1.0_aarch64.dmg",
        "https://github.com/Kevin-hDev/Beaver/releases/download/v1.1.0/Fake_1.1.0_aarch64.dmg",
        "https://github.com/Kevin-hDev/Beaver/releases/download/v1.1.0/Beaver_1.1.0_aarch64.dmg.sha256",
        "https://github.com.evil.test/Kevin-hDev/Beaver/releases/download/v1.1.0/Beaver_1.1.0_aarch64.dmg",
        "https://github.com:443/Kevin-hDev/Beaver/releases/download/v1.1.0/Beaver_1.1.0_aarch64.dmg",
        "https://user@github.com/Kevin-hDev/Beaver/releases/download/v1.1.0/Beaver_1.1.0_aarch64.dmg",
        "https://github.com/Kevin-hDev/Beaver/releases/download/v1.1.0/../asset.dmg",
    ];
    for url in invalid {
        assert!(
            UPDATE_SOURCE.exact_asset_url(url, VERSION, ASSET).is_none(),
            "{url}"
        );
    }
}

#[test]
fn redirects_are_https_bounded_and_cross_to_one_exact_host() {
    let previous = valid_asset_url();
    let target = url::Url::parse(
        "https://release-assets.githubusercontent.com/github-production-release-asset/file?sig=ok",
    )
    .unwrap();
    assert!(release_redirect_is_allowed(&previous, &target, 1));
    assert!(release_redirect_is_allowed(&previous, &target, 2));
    assert!(!release_redirect_is_allowed(&previous, &target, 3));
    let oversized = url::Url::parse(&format!(
        "https://release-assets.githubusercontent.com/{}",
        "x".repeat(2_048)
    ))
    .unwrap();
    assert!(!release_redirect_is_allowed(&previous, &oversized, 1));
    let manifest = UPDATE_SOURCE.manifest_url(VERSION).unwrap();
    assert!(release_redirect_is_allowed(&manifest, &target, 1));

    for raw in [
        "http://release-assets.githubusercontent.com/file",
        "https://release-assets.githubusercontent.com.evil.test/file",
        "https://release-assets.githubusercontent.com:444/file",
        "https://user@release-assets.githubusercontent.com/file",
        "https://objects.githubusercontent.com/file",
    ] {
        let target = url::Url::parse(raw).unwrap();
        assert!(!release_redirect_is_allowed(&previous, &target, 1), "{raw}");
    }
}

#[test]
fn update_requests_never_define_sensitive_headers() {
    let request = update_request(reqwest::Client::new().get(valid_asset_url()))
        .build()
        .expect("request");
    assert!(request.headers().contains_key(USER_AGENT));
    assert!(!request.headers().contains_key(AUTHORIZATION));
    assert!(!request.headers().contains_key(COOKIE));
    assert!(!request.headers().contains_key(PROXY_AUTHORIZATION));
    assert!(download_client().is_ok());
}
