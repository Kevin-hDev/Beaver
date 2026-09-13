use super::launch_args::LaunchContext;
use std::path::PathBuf;

const RUN_ID: &str = "0123456789abcdef0123456789abcdef";
const SHA: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

fn valid_args() -> Vec<String> {
    let version = "1.2.3";
    let asset = if cfg!(target_os = "macos") {
        format!("Beaver_{version}_aarch64.dmg")
    } else {
        format!("Beaver_{version}_x64-setup.exe")
    };
    vec![
        "installer".into(),
        "--run-id".into(),
        RUN_ID.into(),
        "--work-dir".into(),
        std::env::temp_dir()
            .join(format!("beaver-install-{RUN_ID}"))
            .display()
            .to_string(),
        "--version".into(),
        version.into(),
        "--app-asset-name".into(),
        asset,
        "--app-asset-size".into(),
        "42".into(),
        "--app-asset-sha256".into(),
        SHA.into(),
    ]
}

fn parse(args: Vec<String>) -> bool {
    LaunchContext::parse_from(args).is_ok()
}

#[test]
fn accepts_the_exact_pinned_release_contract() {
    let parsed = LaunchContext::parse_from(valid_args()).unwrap();
    assert_eq!(parsed.run_id, RUN_ID);
    assert_eq!(parsed.release.version, "1.2.3");
    assert_eq!(parsed.release.app_asset_size, 42);
    assert!(parsed.work_dir.is_absolute());
}

#[test]
fn rejects_non_canonical_versions_and_unmatched_assets() {
    for invalid in ["01.2.3", "1.2", "1.2.3-beta", "1.02.3"] {
        let mut args = valid_args();
        args[6] = invalid.into();
        assert!(!parse(args), "accepted version {invalid}");
    }

    let mut args = valid_args();
    args[8] = "Beaver_9.9.9_aarch64.dmg".into();
    assert!(!parse(args));
}

#[test]
fn rejects_invalid_sizes_hashes_run_ids_and_paths() {
    for size in ["0", "2147483649", "nope"] {
        let mut args = valid_args();
        args[10] = size.into();
        assert!(!parse(args), "accepted size {size}");
    }
    for sha in [
        "a",
        "A123456789abcdef0123456789abcdef0123456789abcdef0123456789abcde",
    ] {
        let mut args = valid_args();
        args[12] = sha.into();
        assert!(!parse(args), "accepted sha {sha}");
    }
    let mut args = valid_args();
    args[2] = "short".into();
    assert!(!parse(args));

    let mut args = valid_args();
    args[4] = PathBuf::from("relative/run").display().to_string();
    assert!(!parse(args));
    let mut args = valid_args();
    args[4] = std::env::temp_dir()
        .join("parent/../escape")
        .display()
        .to_string();
    assert!(!parse(args));
}

#[test]
fn rejects_duplicate_unknown_missing_and_control_arguments() {
    let mut duplicate = valid_args();
    duplicate.extend(["--run-id".into(), RUN_ID.into()]);
    assert!(!parse(duplicate));

    let mut unknown = valid_args();
    unknown.extend(["--extra".into(), "value".into()]);
    assert!(!parse(unknown));

    let mut missing = valid_args();
    missing.pop();
    assert!(!parse(missing));

    let mut control = valid_args();
    control[6] = "1.2.3\n".into();
    assert!(!parse(control));
}
