use std::ffi::OsString;
use std::path::Path;

use super::{attach_spec, parse_mount_point};

fn plist_output(mount_points: &[&str]) -> Vec<u8> {
    let entities = mount_points
        .iter()
        .map(|mount| {
            let mut entity = plist::Dictionary::new();
            entity.insert(
                "mount-point".to_string(),
                plist::Value::String((*mount).to_string()),
            );
            plist::Value::Dictionary(entity)
        })
        .collect();
    let mut root = plist::Dictionary::new();
    root.insert("system-entities".to_string(), plist::Value::Array(entities));
    let mut output = Vec::new();
    plist::Value::Dictionary(root)
        .to_writer_xml(&mut output)
        .unwrap();
    output
}

#[test]
fn attaches_read_only_with_a_fixed_mount_point_and_no_noverify() {
    let asset = Path::new("/tmp/update.dmg");
    let mount = Path::new("/tmp/beaver-mount");
    let spec = attach_spec(asset, mount);
    assert_eq!(spec.program, Path::new("/usr/bin/hdiutil"));
    assert_eq!(
        spec.args,
        vec![
            OsString::from("attach"),
            asset.as_os_str().to_owned(),
            OsString::from("-nobrowse"),
            OsString::from("-readonly"),
            OsString::from("-mountpoint"),
            mount.as_os_str().to_owned(),
            OsString::from("-plist"),
        ]
    );
    assert!(!format!("{spec:?}").contains("noverify"));
}

#[test]
fn accepts_exactly_one_expected_mount_point() {
    let directory = tempfile::tempdir().unwrap();
    let expected = std::fs::canonicalize(directory.path()).unwrap();
    let expected_text = expected.to_str().unwrap();
    assert!(parse_mount_point(&plist_output(&[expected_text]), &expected).is_ok());
    assert!(parse_mount_point(&plist_output(&[expected_text, expected_text]), &expected).is_err());
    assert!(parse_mount_point(&plist_output(&["/unexpected"]), &expected).is_err());
}
