use serde_json::{json, Value};

use super::*;
use crate::commands::app_update_assets::{expected_asset_name, UpdateArchitecture, UpdatePlatform};
use crate::commands::app_update_manifest::MAX_UPDATE_MANIFEST_BYTES;
use crate::commands::app_update_source::UPDATE_SOURCE;

const REMOTE_VERSION: &str = "99.0.0";

fn expected_name(platform: UpdatePlatform, architecture: UpdateArchitecture) -> String {
    expected_asset_name(&UPDATE_SOURCE, REMOTE_VERSION, platform, architecture)
        .expect("supported asset")
}

fn asset(name: &str, version: &str) -> Value {
    json!({
        "name": name,
        "size": 12,
        "browser_download_url": format!(
            "https://github.com/Kevin-hDev/Beaver/releases/download/v{version}/{name}"
        )
    })
}

fn release_without_manifest(version: &str, assets: Vec<Value>) -> Vec<u8> {
    serde_json::to_vec(&json!({
        "tag_name": format!("v{version}"),
        "name": format!("Beaver v{version}"),
        "published_at": "2026-06-30T12:00:00Z",
        "draft": false,
        "prerelease": false,
        "assets": assets,
    }))
    .expect("release JSON")
}

fn release(version: &str, mut assets: Vec<Value>) -> Vec<u8> {
    assets.push(asset("update-manifest.json", version));
    release_without_manifest(version, assets)
}

fn parse(
    bytes: &[u8],
    current: &str,
    platform: UpdatePlatform,
    architecture: UpdateArchitecture,
) -> Option<AppUpdateInfo> {
    app_update_from_json(bytes, current, platform, architecture)
}

#[test]
fn selects_only_the_exact_platform_and_architecture_asset() {
    let name = expected_name(UpdatePlatform::Linux, UpdateArchitecture::X86_64);
    let bytes = release(
        REMOTE_VERSION,
        vec![
            asset("Beaver_99.0.0_amd64.AppImage", REMOTE_VERSION),
            asset("Fake_99.0.0_amd64.deb", REMOTE_VERSION),
            asset(&name, REMOTE_VERSION),
        ],
    );

    let info = parse(
        &bytes,
        "1.0.2",
        UpdatePlatform::Linux,
        UpdateArchitecture::X86_64,
    )
    .expect("update");

    assert_eq!(info.version, REMOTE_VERSION);
    assert_eq!(info.title.as_deref(), Some("Beaver v99.0.0"));
    assert_eq!(info.published_at.as_deref(), Some("2026-06-30T12:00:00Z"));
    assert!(info.asset_url.ends_with(&name));
    assert_eq!(info.asset_name, name);
    assert_eq!(info.asset_size, 12);
    assert!(info.manifest_url.ends_with("/update-manifest.json"));
    assert!(info.notes_by_locale.is_none());
    let public = serde_json::to_value(&info).unwrap();
    assert!(public.get("assetName").is_none());
    assert!(public.get("assetSize").is_none());
    assert!(public.get("manifestUrl").is_none());
}

#[test]
fn rejects_a_missing_duplicated_or_oversized_manifest_asset() {
    let name = expected_name(UpdatePlatform::Linux, UpdateArchitecture::X86_64);
    let missing = release_without_manifest(REMOTE_VERSION, vec![asset(&name, REMOTE_VERSION)]);
    let duplicated = release(
        REMOTE_VERSION,
        vec![
            asset(&name, REMOTE_VERSION),
            asset("update-manifest.json", REMOTE_VERSION),
        ],
    );
    let mut oversized_manifest = asset("update-manifest.json", REMOTE_VERSION);
    oversized_manifest["size"] = Value::from((MAX_UPDATE_MANIFEST_BYTES + 1) as u64);
    let oversized = release_without_manifest(
        REMOTE_VERSION,
        vec![asset(&name, REMOTE_VERSION), oversized_manifest],
    );

    for bytes in [missing, duplicated, oversized] {
        assert!(parse(
            &bytes,
            "1.0.2",
            UpdatePlatform::Linux,
            UpdateArchitecture::X86_64,
        )
        .is_none());
    }
}

#[test]
fn rejects_a_right_extension_with_the_wrong_name() {
    let bytes = release(
        REMOTE_VERSION,
        vec![asset("Fake_99.0.0_amd64.deb", REMOTE_VERSION)],
    );
    assert!(parse(
        &bytes,
        "1.0.2",
        UpdatePlatform::Linux,
        UpdateArchitecture::X86_64,
    )
    .is_none());
}

#[test]
fn rejects_untrusted_or_duplicated_asset_urls() {
    let name = expected_name(UpdatePlatform::Macos, UpdateArchitecture::Aarch64);
    let mut wrong = asset(&name, REMOTE_VERSION);
    wrong["browser_download_url"] =
        Value::String("https://example.invalid/Beaver_99.0.0_aarch64.dmg".into());
    let bad_url = release(REMOTE_VERSION, vec![wrong]);
    let duplicate = release(
        REMOTE_VERSION,
        vec![asset(&name, REMOTE_VERSION), asset(&name, REMOTE_VERSION)],
    );

    for bytes in [bad_url, duplicate] {
        assert!(parse(
            &bytes,
            "1.0.2",
            UpdatePlatform::Macos,
            UpdateArchitecture::Aarch64,
        )
        .is_none());
    }
}

#[test]
fn rejects_drafts_prereleases_and_non_newer_versions() {
    let name = expected_name(UpdatePlatform::Windows, UpdateArchitecture::X86_64);
    let mut draft: Value =
        serde_json::from_slice(&release(REMOTE_VERSION, vec![asset(&name, REMOTE_VERSION)]))
            .unwrap();
    draft["draft"] = Value::Bool(true);
    let mut prerelease = draft.clone();
    prerelease["draft"] = Value::Bool(false);
    prerelease["prerelease"] = Value::Bool(true);

    for bytes in [
        serde_json::to_vec(&draft).unwrap(),
        serde_json::to_vec(&prerelease).unwrap(),
    ] {
        assert!(parse(
            &bytes,
            "1.0.2",
            UpdatePlatform::Windows,
            UpdateArchitecture::X86_64,
        )
        .is_none());
    }
    for current in [REMOTE_VERSION, "100.0.0"] {
        assert!(parse(
            &release(REMOTE_VERSION, vec![asset(&name, REMOTE_VERSION)]),
            current,
            UpdatePlatform::Windows,
            UpdateArchitecture::X86_64,
        )
        .is_none());
    }
}
