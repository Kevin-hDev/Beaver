use super::runtime_manifest::RuntimeManifest;

#[test]
fn manifest_is_bounded_versioned_and_strict() {
    let body = br#"{"schema_version":1,"implementation":"cpython","major":3,"minor":14,"requirements_sha256":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"}"#;
    let manifest = RuntimeManifest::parse_bounded(body).expect("manifest");

    assert_eq!(manifest.python_major, 3);
    assert_eq!(manifest.python_minor, 14);
    assert!(RuntimeManifest::parse_bounded(&[b'x'; 513]).is_err());
    assert!(RuntimeManifest::parse_bounded(br#"{"schema_version":1,"extra":true}"#).is_err());
    assert!(RuntimeManifest::parse_bounded(br#"{"schema_version":1,"implementation":"cpython","major":3,"minor":14,"requirements_sha256":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","extra":true}"#).is_err());
    assert!(RuntimeManifest::parse_bounded(br#"{"schema_version":1,"schema_version":1,"implementation":"cpython","major":3,"minor":14,"requirements_sha256":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"}"#).is_err());
    for body in [
        &br#"{"schema_version":1,"implementation":"pypy","major":3,"minor":14,"requirements_sha256":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"}"#[..],
        &br#"{"schema_version":1,"implementation":"cpython","major":2,"minor":14,"requirements_sha256":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"}"#[..],
        &br#"{"schema_version":1,"implementation":"cpython","major":3,"minor":9,"requirements_sha256":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"}"#[..],
        &br#"{"schema_version":1,"implementation":"cpython","major":3,"minor":14,"requirements_sha256":"AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"}"#[..],
    ] {
        assert!(RuntimeManifest::parse_bounded(body).is_err());
    }
}
