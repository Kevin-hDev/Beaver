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
    AtomicWriteStage::PermissionsRepaired,
    AtomicWriteStage::Replaced,
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

            let parent = path.parent().unwrap();
            assert!(std::fs::read_dir(parent).unwrap().all(|entry| {
                !entry
                    .unwrap()
                    .file_name()
                    .to_string_lossy()
                    .ends_with(".tmp")
            }));

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
fn first_write_remains_complete_or_absent_at_every_cutpoint() {
    for &cutpoint in STAGES {
        let root = tempfile::tempdir().expect("temporary profile");
        let path = root.path().join("new/config.json");
        let interrupted = std::panic::catch_unwind(|| {
            atomic_write_with_hook(&path, b"new", |stage| {
                assert_ne!(stage, cutpoint, "forced first-write interruption");
            })
            .expect("write before forced interruption");
        });
        assert!(interrupted.is_err());
        let parent = path.parent().unwrap();
        assert!(std::fs::read_dir(parent).unwrap().all(|entry| {
            !entry
                .unwrap()
                .file_name()
                .to_string_lossy()
                .ends_with(".tmp")
        }));
        if path.exists() {
            assert_eq!(std::fs::read(&path).unwrap(), b"new");
        }
        atomic_write(&path, b"new").expect("first write must converge");
        assert_eq!(std::fs::read(path).unwrap(), b"new");
    }
}

#[test]
fn first_write_residue_never_blocks_the_next_generation() {
    let root = tempfile::tempdir().expect("temporary profile");
    let path = root.path().join("config.json");
    let stale = root
        .path()
        .join(".config.json.0123456789abcdef0123456789abcdef.tmp");
    std::fs::write(&stale, b"interrupted").unwrap();

    atomic_write(&path, b"new").expect("random temporary names must remain convergent");
    assert_eq!(std::fs::read(&path).unwrap(), b"new");
    assert!(
        stale.exists(),
        "recent residue may belong to another writer"
    );
}

#[test]
fn first_launch_profile_files_use_the_atomic_store() {
    let source = include_str!("../storage_migration.rs");
    assert!(
        !source.contains("fs::write("),
        "first-launch profile files must not use direct writes"
    );
}

#[test]
fn user_profile_writers_share_the_atomic_store_authority() {
    let sources = [
        ("favorite_models", include_str!("favorite_models.rs")),
        (
            "personality_injection",
            include_str!("personality_injection.rs"),
        ),
        (
            "forecast_model_config",
            include_str!("forecast/model_config/storage.rs"),
        ),
        (
            "agent_settings",
            include_str!("agent_local/agent_settings.rs"),
        ),
        (
            "project_store",
            include_str!("agent_local/project_store.rs"),
        ),
        (
            "session_permission_state",
            include_str!("agent_local/session_permission_state.rs"),
        ),
        (
            "session_tabs",
            include_str!("agent_local/session_tabs_file.rs"),
        ),
        (
            "subagent_change_store",
            include_str!("agent_local/subagent_change_store.rs"),
        ),
        (
            "tool_plan",
            include_str!("agent_local/tool_plan_storage.rs"),
        ),
        (
            "translation_cache",
            include_str!("agent_local/translation_cache.rs"),
        ),
    ];
    for (name, source) in sources {
        assert!(
            source.contains("private_store::atomic_write"),
            "persistent writer {name} bypasses the atomic store"
        );
    }
    assert!(
        include_str!("agent_local/subagent_startup_cleanup.rs")
            .contains("session_store::write_to_dir"),
        "session cleanup must use the session document authority"
    );

    for (name, source, lock, private_writer) in [
        (
            "favorites",
            include_str!("favorite_models.rs"),
            "FAVORITES_LOCK",
            "fn write_atomic(",
        ),
        (
            "personality",
            include_str!("personality_injection.rs"),
            "INJECTION_LOCK",
            "fn write_state_unlocked(",
        ),
        (
            "forecast",
            include_str!("forecast/model_config/storage.rs"),
            "CONFIG_LOCK",
            "fn write_all(",
        ),
        (
            "agent",
            include_str!("agent_local/agent_settings.rs"),
            "SETTINGS_LOCK",
            "fn save(",
        ),
        (
            "projects",
            include_str!("agent_local/project_store.rs"),
            "PROJECT_STORE_LOCK",
            "fn write_atomic(",
        ),
    ] {
        assert!(
            source.contains(lock),
            "RMW writer {name} lacks its owner lock"
        );
        assert!(
            source.contains(private_writer)
                && !source.lines().any(|line| {
                    let line = line.trim_start();
                    line.starts_with("pub") && line.contains(private_writer)
                }),
            "RMW writer {name} exposes its lock-free persistence helper"
        );
    }
}

#[test]
fn every_fallible_step_runs_before_the_generation_is_published() {
    // A reader can consume the published file at once: the update helper deletes
    // the health acknowledgement the moment it appears. Any fallible step placed
    // after `Replaced` would then act on a file it no longer owns and could turn
    // an acquired publication into a write failure. 1.1.3 repaired the ACL after
    // the rename; this test fails if that order ever comes back.
    let root = tempfile::tempdir().expect("temporary profile");
    let path = root.path().join("update-health").join("token.ok");
    let mut stages = Vec::new();
    atomic_write_with_hook(&path, b"ok", |stage| stages.push(stage)).expect("publication");

    let position = |wanted: AtomicWriteStage| {
        stages
            .iter()
            .position(|stage| *stage == wanted)
            .unwrap_or_else(|| panic!("stage {wanted:?} never reached"))
    };
    assert!(
        position(AtomicWriteStage::PermissionsRepaired) < position(AtomicWriteStage::Replaced),
        "permissions must be repaired on the temporary file, before publication: {stages:?}"
    );
    assert_eq!(
        stages.last(),
        Some(&AtomicWriteStage::ParentSynced),
        "only the parent synchronization may follow publication: {stages:?}"
    );
}
