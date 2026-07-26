use serde_json::{json, Value};
use sha2::{Digest, Sha256};

use super::*;

const VERSION: &str = "1.1.0";
const NAME: &str = "Beaver_1.1.0_aarch64.dmg";
const SIZE: u64 = 12;
const HASH: &str = "a948904f2f0f479b8f8197694b30184b0d2ed1c1cd2a1ec0fb85d299a192a447";

fn asset(name: &str) -> Value {
    json!({ "name": name, "sha256": HASH, "size": SIZE })
}

fn manifest(version: &str, assets: Vec<Value>) -> Vec<u8> {
    serde_json::to_vec(&json!({ "version": version, "assets": assets })).unwrap()
}

#[test]
fn accepts_an_exact_bounded_manifest() {
    let parsed =
        parse_update_manifest(&manifest(VERSION, vec![asset(NAME)]), VERSION).expect("manifest");
    let selected = parsed.asset(NAME, SIZE).expect("declared asset");

    assert_eq!(selected.name, NAME);
    assert_eq!(selected.size, SIZE);
    assert_eq!(selected.sha256, HASH);
    assert!(parsed.asset(NAME, SIZE + 1).is_none());
    assert!(parsed.asset("Beaver_1.1.0_x64.dmg", SIZE).is_none());
}

#[test]
fn rejects_absent_malformed_and_oversized_manifests() {
    assert!(parse_update_manifest(b"", VERSION).is_none());
    assert!(parse_update_manifest(b"{", VERSION).is_none());

    let oversized = vec![b' '; MAX_UPDATE_MANIFEST_BYTES + 1];
    assert!(parse_update_manifest(&oversized, VERSION).is_none());
}

#[test]
fn rejects_too_many_or_duplicated_assets() {
    let too_many = (0..=MAX_UPDATE_MANIFEST_ASSETS)
        .map(|index| asset(&format!("Beaver_1.1.0_asset-{index}.dmg")))
        .collect();
    assert!(parse_update_manifest(&manifest(VERSION, too_many), VERSION).is_none());

    let duplicate = manifest(VERSION, vec![asset(NAME), asset(NAME)]);
    assert!(parse_update_manifest(&duplicate, VERSION).is_none());
}

#[test]
fn rejects_invalid_hash_name_and_size() {
    let invalid = [
        json!({ "name": NAME, "sha256": "0".repeat(63), "size": SIZE }),
        json!({ "name": NAME, "sha256": "G".repeat(64), "size": SIZE }),
        json!({ "name": "../Beaver.dmg", "sha256": HASH, "size": SIZE }),
        json!({ "name": NAME, "sha256": HASH, "size": 0 }),
        json!({ "name": NAME, "sha256": HASH, "size": MAX_UPDATE_ASSET_BYTES + 1 }),
    ];

    for entry in invalid {
        assert!(parse_update_manifest(&manifest(VERSION, vec![entry]), VERSION).is_none());
    }
}

#[test]
fn rejects_a_version_different_from_the_release_tag() {
    assert!(parse_update_manifest(&manifest("1.1.1", vec![asset(NAME)]), VERSION).is_none());
    assert!(parse_update_manifest(&manifest("../1.1.0", vec![asset(NAME)]), VERSION).is_none());
}

#[test]
fn compares_sha256_bytes_in_constant_time() {
    let actual: [u8; 32] = Sha256::digest(b"hello world\n").into();
    assert!(sha256_matches(&actual, HASH));
    assert!(!sha256_matches(&actual, &"0".repeat(64)));
    assert!(!sha256_matches(&actual, "invalid"));
}

#[test]
fn download_size_is_bounded_even_without_content_length() {
    assert_eq!(
        checked_download_size(MAX_UPDATE_ASSET_BYTES - 1, 1, MAX_UPDATE_ASSET_BYTES),
        Some(MAX_UPDATE_ASSET_BYTES)
    );
    assert_eq!(
        checked_download_size(MAX_UPDATE_ASSET_BYTES, 1, MAX_UPDATE_ASSET_BYTES),
        None
    );
    assert_eq!(checked_download_size(10, 3, 12), None);
}
