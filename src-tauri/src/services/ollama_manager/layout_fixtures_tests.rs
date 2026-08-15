use super::fingerprint::{BundleFingerprint, OllamaVersion, Sha256Digest};
use super::recovery_decision::{
    decide_recovery, ArchiveDirectoryEvidence, DirectoryEvidence, JournalPresence,
    MigrationMarkerPresence, OllamaLayoutSnapshot, RecoveryDecision,
};
use crate::services::paths::ollama_paths;
use std::fs;

const VERSIONS: [&str; 3] = ["1.1.0", "1.1.1", "1.1.2"];
const PLATFORMS: [&str; 3] = ["macos", "linux", "windows"];

fn fixture_fingerprint(version: &str) -> BundleFingerprint {
    let byte = match version {
        "1.1.0" => "11",
        "1.1.1" => "22",
        "1.1.2" => "33",
        _ => unreachable!("layout fixture version"),
    };
    BundleFingerprint {
        version: OllamaVersion::parse(version).expect("layout version"),
        executable_sha256: Sha256Digest::from_hex(&byte.repeat(32)).expect("layout digest"),
    }
}

fn empty_snapshot() -> OllamaLayoutSnapshot {
    OllamaLayoutSnapshot {
        journal: JournalPresence::Absent,
        migration_marker: MigrationMarkerPresence::Absent,
        active: DirectoryEvidence::Absent,
        install_staging: DirectoryEvidence::Absent,
        archive_staging: ArchiveDirectoryEvidence::Absent,
        archive_failed: ArchiveDirectoryEvidence::Absent,
        update_staging: DirectoryEvidence::Absent,
        backup: DirectoryEvidence::Absent,
        failed: DirectoryEvidence::Absent,
        legacy_staging: DirectoryEvidence::Absent,
        legacy_backup: DirectoryEvidence::Absent,
        backup_delete: DirectoryEvidence::Absent,
        failed_delete: DirectoryEvidence::Absent,
    }
}

#[test]
fn every_beaver_layout_fixture_converges_without_touching_models() {
    let mut count = 0;
    for version in VERSIONS {
        for platform in PLATFORMS {
            count += 1;
            let root = tempfile::tempdir().expect("fixture root");
            let paths = ollama_paths(root.path());
            let models = root.path().join(format!("models-{platform}"));
            fs::create_dir_all(&models).expect("models directory");
            fs::write(
                models.join("manifest.json"),
                format!("{version}:{platform}"),
            )
            .unwrap();
            for (path, name) in [
                (&paths.active, "ollama-bundle"),
                (&paths.legacy_staging, "ollama-bundle-staging"),
                (&paths.legacy_backup, "ollama-bundle-old"),
                (&paths.failed, "ollama-bundle-failed"),
            ] {
                assert_eq!(path.parent(), Some(root.path()));
                assert_eq!(
                    path.file_name().and_then(|value| value.to_str()),
                    Some(name)
                );
                fs::create_dir_all(path).expect("published layout directory");
                fs::remove_dir(path).expect("remove unused layout directory");
            }
            fs::create_dir_all(&paths.legacy_backup).expect("legacy backup");
            fs::write(paths.legacy_backup.join("VERSION"), version).unwrap();

            let fingerprint = fixture_fingerprint(version);
            let mut snapshot = empty_snapshot();
            snapshot.legacy_backup = DirectoryEvidence::Present(fingerprint.clone());
            assert_eq!(
                decide_recovery(&snapshot),
                RecoveryDecision::RestoreLegacyBackup,
                "legacy layout {version} on {platform}"
            );

            fs::rename(&paths.legacy_backup, &paths.active).expect("legacy adoption rename");
            snapshot.legacy_backup = DirectoryEvidence::Absent;
            snapshot.active = DirectoryEvidence::Present(fingerprint);
            assert_eq!(
                decide_recovery(&snapshot),
                RecoveryDecision::AdoptLegacyActive,
                "marker-free active layout {version} on {platform}"
            );

            snapshot.migration_marker = MigrationMarkerPresence::Valid(Default::default());
            assert_eq!(decide_recovery(&snapshot), RecoveryDecision::Ready);
            assert_eq!(
                fs::read(models.join("manifest.json")).unwrap(),
                format!("{version}:{platform}").as_bytes()
            );
        }
    }
    assert_eq!(count, 9);
}

#[test]
fn layout_fixture_matrix_has_three_versions_and_three_platforms() {
    assert_eq!(VERSIONS, ["1.1.0", "1.1.1", "1.1.2"]);
    assert_eq!(PLATFORMS, ["macos", "linux", "windows"]);
}
