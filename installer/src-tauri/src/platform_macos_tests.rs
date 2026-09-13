use super::platform::macos::{
    validate_bundle, validate_destination, validate_staged_bundle, validate_target,
};
use super::platform::macos_authorization::{validate_call, AuthorizationScope, ProtectedTool};
use super::platform::macos_dmg::{install_dmg, MacInstallResult};
use super::platform::macos_install::swap_unprivileged;
use std::fs;
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT: AtomicU64 = AtomicU64::new(1);

struct Fixture(PathBuf);

impl Fixture {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "beaver-macos-install-test-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&root).unwrap();
        Self(root)
    }

    fn bundle(&self) -> PathBuf {
        let bundle = self.0.join("Beaver.app");
        fs::create_dir_all(bundle.join("Contents/MacOS")).unwrap();
        fs::write(
            bundle.join("Contents/Info.plist"),
            r#"<?xml version="1.0" encoding="UTF-8"?>
<plist version="1.0"><dict>
<key>CFBundleIdentifier</key><string>com.clgo.dash</string>
<key>CFBundleExecutable</key><string>cl-go-dash</string>
<key>CFBundleShortVersionString</key><string>1.2.2</string>
</dict></plist>"#,
        )
        .unwrap();
        fs::write(bundle.join("Contents/MacOS/cl-go-dash"), "binary").unwrap();
        bundle
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn validates_local_custom_destinations_and_target_types() {
    let fixture = Fixture::new();
    let destination = validate_destination(&fixture.0).unwrap();
    assert!(!destination.needs_authorization());
    assert!(validate_target(&destination).is_ok());

    let outside = Fixture::new();
    let link = fixture.0.join("link");
    symlink(&outside.0, &link).unwrap();
    assert!(validate_destination(&link).is_err());

    symlink(&outside.0, fixture.0.join("Beaver.app")).unwrap();
    assert!(validate_target(&destination).is_err());
}

#[test]
fn validates_only_the_expected_bundle_inside_the_mount() {
    let fixture = Fixture::new();
    let bundle = fixture.bundle();
    let validated = validate_bundle(&bundle, &fixture.0).unwrap();
    assert_eq!(validated.root(), bundle.canonicalize().unwrap());

    fs::write(
        bundle.join("Contents/Info.plist"),
        r#"<plist><dict><key>CFBundleIdentifier</key><string>evil.app</string><key>CFBundleExecutable</key><string>cl-go-dash</string></dict></plist>"#,
    )
    .unwrap();
    assert!(validate_bundle(&bundle, &fixture.0).is_err());
}

#[test]
fn privileged_calls_are_limited_to_apple_tools_and_app_siblings() {
    let source = Path::new("/Volumes/Beaver/Beaver.app");
    let scope =
        AuthorizationScope::new(Path::new("/Applications"), Path::new("/Volumes/Beaver")).unwrap();
    assert!(validate_call(
        ProtectedTool::Ditto,
        &[
            source.into(),
            PathBuf::from("/Applications/.Beaver.app.stage-0123456789abcdef0123456789abcdef")
        ],
        &scope,
    )
    .is_ok());
    assert!(validate_call(
        ProtectedTool::Move,
        &[
            PathBuf::from("/Applications/Beaver.app"),
            PathBuf::from("/Applications/.Beaver.app.backup-0123456789abcdef0123456789abcdef"),
        ],
        &scope,
    )
    .is_ok());
    for (tool, args) in [
        (ProtectedTool::Move, vec![PathBuf::from("--help")]),
        (
            ProtectedTool::Remove,
            vec![PathBuf::from("/tmp/Beaver.app")],
        ),
        (
            ProtectedTool::Ditto,
            vec![
                PathBuf::from("/tmp/Beaver.app"),
                PathBuf::from("/Applications/Beaver.app"),
            ],
        ),
    ] {
        assert!(validate_call(tool, &args, &scope).is_err());
    }
    assert!(AuthorizationScope::new(Path::new("/tmp"), Path::new("/Volumes/Beaver")).is_err());
}

#[test]
fn swaps_complete_bundles_and_removes_the_backup() {
    let destination = Fixture::new();
    let old = destination.bundle();
    fs::write(old.join("Contents/MacOS/cl-go-dash"), "old").unwrap();

    let staged_root = Fixture::new();
    let stage_source = staged_root.bundle();
    fs::write(stage_source.join("Contents/MacOS/cl-go-dash"), "new").unwrap();
    let stage = destination
        .0
        .join(".Beaver.app.stage-0123456789abcdef0123456789abcdef");
    fs::rename(stage_source, &stage).unwrap();
    let validated = validate_destination(&destination.0).unwrap();
    let mut non_return_published = false;

    swap_unprivileged(&stage, &validated, "1.2.2", || {
        non_return_published = true;
        Ok(())
    })
    .unwrap();

    assert!(non_return_published);
    assert_eq!(
        fs::read(destination.0.join("Beaver.app/Contents/MacOS/cl-go-dash")).unwrap(),
        b"new"
    );
    assert_eq!(fs::read_dir(&destination.0).unwrap().count(), 1);
}

#[test]
fn refuses_to_publish_a_bundle_with_the_wrong_version() {
    let destination = Fixture::new();
    let old = destination.bundle();
    fs::write(old.join("Contents/MacOS/cl-go-dash"), "old").unwrap();
    let staged_root = Fixture::new();
    let stage = destination
        .0
        .join(".Beaver.app.stage-0123456789abcdef0123456789abcdef");
    fs::rename(staged_root.bundle(), &stage).unwrap();
    let validated = validate_destination(&destination.0).unwrap();

    assert!(swap_unprivileged(&stage, &validated, "9.9.9", || Ok(())).is_err());
    assert_eq!(
        fs::read(destination.0.join("Beaver.app/Contents/MacOS/cl-go-dash")).unwrap(),
        b"old"
    );
}

#[test]
fn refuses_an_unowned_staging_sibling_name() {
    let destination = Fixture::new();
    let source = Fixture::new();
    let arbitrary = destination.0.join("unowned.app");
    fs::rename(source.bundle(), &arbitrary).unwrap();
    let validated = validate_destination(&destination.0).unwrap();
    assert!(validate_staged_bundle(&arbitrary, &validated).is_err());
}

#[test]
#[ignore = "mounts a local DMG and exercises the real macOS installer"]
fn installs_a_complete_bundle_from_a_local_dmg() {
    let work = Fixture::new();
    let source = Fixture::new();
    fs::write(source.bundle().join("new-only"), "new").unwrap();
    let asset = work.0.join("Beaver_1.2.2_aarch64.dmg");
    let status = std::process::Command::new("/usr/bin/hdiutil")
        .args(["create", "-format", "UDZO", "-srcfolder"])
        .arg(&source.0)
        .arg(&asset)
        .env_clear()
        .status()
        .unwrap();
    assert!(status.success());

    let destination = Fixture::new();
    fs::write(destination.bundle().join("old-only"), "old").unwrap();
    let validated = validate_destination(&destination.0).unwrap();
    let mut began_swap = false;
    let result = install_dmg(
        &asset,
        &work.0,
        &validated,
        &tokio_util::sync::CancellationToken::new(),
        "1.2.2",
        || {
            began_swap = true;
            Ok(())
        },
    )
    .unwrap();

    assert_eq!(result, MacInstallResult::Reinstalled);
    assert!(began_swap);
    assert!(destination.0.join("Beaver.app/new-only").is_file());
    assert!(!destination.0.join("Beaver.app/old-only").exists());
}
