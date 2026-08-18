use super::atomic_write;
use super::private_store_atomic::{atomic_write_with_hook, AtomicWriteStage};

const PROFILE_FILES: &[(&str, &[u8], &[u8])] = &[
    (
        "config.json",
        br#"{"generation":"old"}"#,
        br#"{"generation":"new"}"#,
    ),
    (
        "agent-sessions/session.json",
        br#"{"messages":["old"]}"#,
        br#"{"messages":["new"]}"#,
    ),
    ("secrets.enc", b"old-test-vault", b"new-test-vault"),
    (
        "forecast-analyses/analysis.json",
        br#"{"result":"old"}"#,
        br#"{"result":"new"}"#,
    ),
    ("skills/test/SKILL.md", b"old skill\n", b"new skill\n"),
    ("memory/core/user.md", b"old memory\n", b"new memory\n"),
    (
        "ollama-custom-models.json",
        br#"[{"name":"old"}]"#,
        br#"[{"name":"new"}]"#,
    ),
];

const STAGES: &[AtomicWriteStage] = &[
    AtomicWriteStage::TempOpened,
    AtomicWriteStage::ContentWritten,
    AtomicWriteStage::FileSynced,
    AtomicWriteStage::Replaced,
    AtomicWriteStage::PermissionsRepaired,
    AtomicWriteStage::ParentSynced,
];

#[test]
fn representative_profile_reopens_after_every_atomic_write_cutpoint() {
    for &(relative, old, new) in PROFILE_FILES {
        for &cutpoint in STAGES {
            let root = tempfile::tempdir().expect("temporary profile");
            let path = root.path().join(relative);
            atomic_write(&path, old).expect("seed old profile value");

            let interrupted = std::panic::catch_unwind(|| {
                atomic_write_with_hook(&path, new, |stage| {
                    assert_ne!(stage, cutpoint, "forced profile interruption");
                })
                .expect("write before forced interruption");
            });
            assert!(interrupted.is_err(), "cutpoint must interrupt the write");

            let reopened = std::fs::read(&path).expect("reopen profile value");
            assert!(
                reopened == old || reopened == new,
                "profile value must remain an entire generation"
            );
            if relative.ends_with(".json") {
                serde_json::from_slice::<serde_json::Value>(&reopened)
                    .expect("reopened JSON must remain complete");
            }

            atomic_write(&path, new).expect("subsequent write must converge");
            assert_eq!(std::fs::read(path).unwrap(), new);
        }
    }
}

#[test]
fn first_launch_profile_files_use_the_atomic_store() {
    let source = include_str!("../storage_migration.rs");
    assert!(
        !source.contains("fs::write("),
        "first-launch profile files must not use direct writes"
    );
}
