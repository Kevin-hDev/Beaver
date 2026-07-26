use serde_json::{json, Value};

use super::*;
use crate::commands::app_update_assets::{UpdateArchitecture, UpdatePlatform, MAX_RELEASE_ASSETS};
use crate::commands::app_update_source::is_safe_version;

const VERSION: &str = "99.0.0";

fn asset(name: &str) -> Value {
    json!({
        "name": name,
        "size": 12,
        "browser_download_url": format!(
            "https://github.com/Kevin-hDev/Beaver/releases/download/v{VERSION}/{name}"
        )
    })
}

fn release(mut assets: Vec<Value>) -> Vec<u8> {
    assets.push(asset("update-manifest.json"));
    serde_json::to_vec(&json!({
        "tag_name": format!("v{VERSION}"),
        "name": format!("Beaver v{VERSION}"),
        "published_at": "2026-06-30T12:00:00Z",
        "draft": false,
        "prerelease": false,
        "assets": assets,
    }))
    .unwrap()
}

#[test]
fn rejects_oversized_responses_and_asset_collections() {
    let too_large = vec![b' '; MAX_RELEASE_RESPONSE_BYTES + 1];
    assert!(app_update_from_json(
        &too_large,
        "1.0.2",
        UpdatePlatform::Linux,
        UpdateArchitecture::X86_64,
    )
    .is_none());

    let assets = (0..=MAX_RELEASE_ASSETS)
        .map(|index| asset(&format!("Other_{index}.txt")))
        .collect();
    assert!(app_update_from_json(
        &release(assets),
        "1.0.2",
        UpdatePlatform::Linux,
        UpdateArchitecture::X86_64,
    )
    .is_none());
}

#[test]
fn versions_are_strict_three_part_numbers() {
    assert!(is_safe_version("1.1.0"));
    assert!(version_gt("0.12.4", "0.12.3"));
    assert!(!version_gt(&"1".repeat(65), "0.12.3"));
    for version in ["", "1.1", "1.1.0.0", "01.1.0", "1.1.0-beta", "../1.1.0"] {
        assert!(!is_safe_version(version), "{version}");
    }
}
